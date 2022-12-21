import { Client } from "./lib";
const config: RTCConfiguration = {
  iceServers: [
    {
      urls: ["stun:stun.l.google.com:19302", "stun:openrelay.metered.ca:80"],
    },
    // OpenRelay TURN servers. Used to make stun/turn work in gitpod and github codespaces
    // In the actual robot, `coturn` is needed.
    {
      urls: "turn:openrelay.metered.ca:80",
      username: "openrelayproject",
      credential: "openrelayproject",
    },
    {
      urls: "turn:openrelay.metered.ca:443",
      username: "openrelayproject",
      credential: "openrelayproject",
    },
    {
      urls: "turn:openrelay.metered.ca:443?transport=tcp",
      username: "openrelayproject",
      credential: "openrelayproject",
    },
  ],
};

// The local browser's RTCPeerConnection
const peerConnection = new RTCPeerConnection(config);

const video = document.querySelector<HTMLVideoElement>("#localVideo")!;

async function main() {
  const url = location.origin.replace("1234", "8080");

  console.log(`Connecting to url ${url}.`);

  const client = new Client(url);

  let mediaStream: MediaStream;

  peerConnection.setLocalDescription(
    await peerConnection.createOffer({
      offerToReceiveAudio: true,
      offerToReceiveVideo: true,
    })
  );

  console.log("Beginning transmission");

  peerConnection.addEventListener("track", (event) => {
    console.log("Found Track: ", event.streams[0]);
    mediaStream = event.streams[0];
    video.srcObject = mediaStream;
  });

  peerConnection.addEventListener("icecandidate", async ({ candidate }) => {
    console.log(candidate);
    if (candidate == null && peerConnection.localDescription != null) {
      const answer = await client.watch(peerConnection.localDescription);
      console.log(answer);
      peerConnection.setRemoteDescription(answer);
    } else if (candidate == null) {
      throw new Error("Local description is null! This should not happen.");
    }
  });
}

main();
