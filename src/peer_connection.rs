use anyhow::{anyhow, Result};
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::mpsc::Receiver;
use tokio::sync::Notify;
use tokio::time::Duration;
use webrtc::api::interceptor_registry::register_default_interceptors;
use webrtc::api::media_engine::{MediaEngine, MIME_TYPE_H264, MIME_TYPE_OPUS};
use webrtc::api::APIBuilder;
use webrtc::ice_transport::ice_connection_state::RTCIceConnectionState;
use webrtc::ice_transport::ice_server::RTCIceServer;
use webrtc::interceptor::registry::Registry;
use webrtc::media::io::h264_reader::H264Reader;
use webrtc::media::io::ogg_reader::OggReader;
use webrtc::media::Sample;
use webrtc::peer_connection::configuration::RTCConfiguration;
use webrtc::peer_connection::peer_connection_state::RTCPeerConnectionState;
use webrtc::peer_connection::sdp::session_description::RTCSessionDescription;
use webrtc::peer_connection::RTCPeerConnection;
use webrtc::rtp_transceiver::rtp_codec::RTCRtpCodecCapability;
use webrtc::track::track_local::track_local_static_sample::TrackLocalStaticSample;
use webrtc::track::track_local::TrackLocal;
use webrtc::Error;

use crate::media_type::MediaType;

const OGG_PAGE_DURATION: Duration = Duration::from_millis(20);

pub struct PeerConnectionInfo {
    pub connection: Arc<RTCPeerConnection>,
    pub description: RTCSessionDescription,
    pub closer: Receiver<()>,
}

pub async fn connect(
    media_type: MediaType,
    offer: &str,
) -> Result<PeerConnectionInfo> {

    // TODO support audio
    let audio: Option<String> = None;

    if let MediaType::File(video_path) = &media_type {
        if !Path::new(video_path).exists() {
            return Err(Error::new(format!("video file: '{video_path}' not exist")).into());
        }
    }

    let video = match media_type {
        MediaType::File(video_path) => Some(video_path),
        MediaType::Video4Linux2(video_path) => Some(video_path),
    };

    if let Some(audio_path) = &audio {
        if !Path::new(audio_path).exists() {
            return Err(Error::new(format!("audio file: '{audio_path}' not exist")).into());
        }
    }

    // Everything below is the WebRTC-rs API! Thanks for using it ❤️.

    // Create a MediaEngine object to configure the supported codec
    let mut m = MediaEngine::default();

    m.register_default_codecs()?;

    // Create a InterceptorRegistry. This is the user configurable RTP/RTCP Pipeline.
    // This provides NACKs, RTCP Reports and other features. If you use `webrtc.NewPeerConnection`
    // this is enabled by default. If you are manually managing You MUST create a InterceptorRegistry
    // for each PeerConnection.
    let mut registry = Registry::new();

    // Use the default set of Interceptors
    registry = register_default_interceptors(registry, &mut m)?;

    // Create the API object with the MediaEngine
    let api = APIBuilder::new()
        .with_media_engine(m)
        .with_interceptor_registry(registry)
        .build();

    // Prepare the configuration
    let config = RTCConfiguration {
        ice_servers: vec![RTCIceServer {
            urls: vec!["stun:stun.l.google.com:19302".to_owned()],
            ..Default::default()
        }],
        ..Default::default()
    };

    // Create a new RTCPeerConnection
    let peer_connection = Arc::new(api.new_peer_connection(config).await?);

    let notify_tx = Arc::new(Notify::new());
    let notify_video = notify_tx.clone();
    let notify_audio = notify_tx.clone();

    let (done_tx, done_rx) = tokio::sync::mpsc::channel::<()>(1);
    let video_done_tx = done_tx.clone();
    let audio_done_tx = done_tx.clone();

    if let Some(video_file) = video {
        // Create a video track
        let video_track = Arc::new(TrackLocalStaticSample::new(
            RTCRtpCodecCapability {
                mime_type: MIME_TYPE_H264.to_owned(),
                ..Default::default()
            },
            "video".to_owned(),
            "webrtc-rs".to_owned(),
        ));

        // Add this newly created track to the PeerConnection
        let rtp_sender = peer_connection
            .add_track(Arc::clone(&video_track) as Arc<dyn TrackLocal + Send + Sync>)
            .await?;

        // Read incoming RTCP packets
        // Before these packets are returned they are processed by interceptors. For things
        // like NACK this needs to be called.
        tokio::spawn(async move {
            let mut rtcp_buf = vec![0u8; 1500];
            while let Ok((_, _)) = rtp_sender.read(&mut rtcp_buf).await {}
            Result::<()>::Ok(())
        });

        let video_file_name = video_file.clone();
        tokio::spawn(async move {
            // Open a H264 file and start reading using our H264Reader
            let file = File::open(&video_file_name)?;
            let reader = BufReader::new(file);
            let mut h264 = H264Reader::new(reader);

            // Wait for connection established
            notify_video.notified().await;

            debug!("play video from disk file {video_file_name}");

            // It is important to use a time.Ticker instead of time.Sleep because
            // * avoids accumulating skew, just calling time.Sleep didn't compensate for the time spent parsing the data
            // * works around latency issues with Sleep
            let mut ticker = tokio::time::interval(Duration::from_millis(33));
            loop {
                let nal = match h264.next_nal() {
                    Ok(nal) => nal,
                    Err(err) => {
                        debug!("All video frames parsed and sent: {err}");
                        break;
                    }
                };

                video_track
                    .write_sample(&Sample {
                        data: nal.data.freeze(),
                        duration: Duration::from_secs(1),
                        ..Default::default()
                    })
                    .await?;

                let _ = ticker.tick().await;
            }

            let _ = video_done_tx.try_send(());

            Result::<()>::Ok(())
        });
    }

    if let Some(audio_file) = audio {
        // Create a audio track
        let audio_track = Arc::new(TrackLocalStaticSample::new(
            RTCRtpCodecCapability {
                mime_type: MIME_TYPE_OPUS.to_owned(),
                ..Default::default()
            },
            "audio".to_owned(),
            "webrtc-rs".to_owned(),
        ));

        // Add this newly created track to the PeerConnection
        let rtp_sender = peer_connection
            .add_track(Arc::clone(&audio_track) as Arc<dyn TrackLocal + Send + Sync>)
            .await?;

        // Read incoming RTCP packets
        // Before these packets are returned they are processed by interceptors. For things
        // like NACK this needs to be called.
        tokio::spawn(async move {
            let mut rtcp_buf = vec![0u8; 1500];
            while let Ok((_, _)) = rtp_sender.read(&mut rtcp_buf).await {}
            Result::<()>::Ok(())
        });

        let audio_file_name = audio_file.clone();
        tokio::spawn(async move {
            // Open a IVF file and start reading using our IVFReader
            let file = File::open(audio_file_name)?;
            let reader = BufReader::new(file);
            // Open on oggfile in non-checksum mode.
            let (mut ogg, _) = OggReader::new(reader, true)?;

            // Wait for connection established
            notify_audio.notified().await;

            debug!("play audio from disk file output.ogg");

            // It is important to use a time.Ticker instead of time.Sleep because
            // * avoids accumulating skew, just calling time.Sleep didn't compensate for the time spent parsing the data
            // * works around latency issues with Sleep
            let mut ticker = tokio::time::interval(OGG_PAGE_DURATION);

            // Keep track of last granule, the difference is the amount of samples in the buffer
            let mut last_granule: u64 = 0;
            while let Ok((page_data, page_header)) = ogg.parse_next_page() {
                // The amount of samples is the difference between the last and current timestamp
                let sample_count = page_header.granule_position - last_granule;
                last_granule = page_header.granule_position;
                let sample_duration = Duration::from_millis(sample_count * 1000 / 48000);

                audio_track
                    .write_sample(&Sample {
                        data: page_data.freeze(),
                        duration: sample_duration,
                        ..Default::default()
                    })
                    .await?;

                let _ = ticker.tick().await;
            }

            let _ = audio_done_tx.try_send(());

            Result::<()>::Ok(())
        });
    }

    // Set the handler for ICE connection state
    // This will notify you when the peer has connected/disconnected
    peer_connection.on_ice_connection_state_change(Box::new(
        move |connection_state: RTCIceConnectionState| {
            debug!("Connection State has changed {connection_state}");
            if connection_state == RTCIceConnectionState::Connected {
                notify_tx.notify_waiters();
            }
            Box::pin(async {})
        },
    ));

    // Set the handler for Peer connection state
    // This will notify you when the peer has connected/disconnected
    peer_connection.on_peer_connection_state_change(Box::new(
        move |state: RTCPeerConnectionState| {
            debug!("Peer Connection State has changed: {state}",);

            if state == RTCPeerConnectionState::Failed {
                // Wait until PeerConnection has had no network activity for 30 seconds or another failure. It may be reconnected using an ICE Restart.
                // Use webrtc.PeerConnectionStateDisconnected if you are interested in detecting faster timeout.
                // Note that the PeerConnection may come back from PeerConnectionStateDisconnected.
                debug!("Peer Connection has gone to failed -- exiting now");
                let _ = done_tx.try_send(());
            }

            if state == RTCPeerConnectionState::Closed {
                debug!("Peer Connection has gone to closed -- exiting now");
                let _ = done_tx.try_send(());
            }

            Box::pin(async {})
        },
    ));

    // Wait for the offer to be pasted
    let offer = serde_json::from_str::<RTCSessionDescription>(offer)?;

    // Set the remote SessionDescription
    peer_connection.set_remote_description(offer).await?;

    // Create an answer
    let answer = peer_connection.create_answer(None).await?;

    // Create channel that is blocked until ICE Gathering is complete
    let mut gather_complete = peer_connection.gathering_complete_promise().await;

    // Sets the LocalDescription, and starts our UDP listeners
    peer_connection.set_local_description(answer).await?;

    // Block until ICE Gathering is complete, disabling trickle ICE
    // we do this because we only can exchange one signaling message
    // in a production application you should exchange ICE Candidates via OnICECandidate
    let _ = gather_complete.recv().await;

    if let Some(local_desc) = peer_connection.local_description().await {
        Ok(PeerConnectionInfo {
            connection: peer_connection,
            description: local_desc,
            closer: done_rx,
        })
    } else {
        Err(anyhow!("Failed to get local description"))
    }
}

pub async fn handle(mut info: PeerConnectionInfo) -> Result<()> {
    tokio::select! {
        _ = info.closer.recv() => {
            trace!("received done signal!");
        }
        _ = tokio::signal::ctrl_c() => {
            println!();
        }
    };

    info.connection.close().await?;

    Ok(())
}
