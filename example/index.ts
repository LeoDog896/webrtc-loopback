import { Client } from "./lib"

const messageTextBox = document.querySelector<HTMLInputElement>("message")!;

const config = { iceServers: [{ urls: "stun:stun.l.google.com:19302" }] };

// The local browser's RTCPeerConnection
const peerConnection = new RTCPeerConnection(config);

const video = document.querySelector<HTMLVideoElement>("localVideo")!;

async function main() {
    const client = new Client("http://localhost:8080")

    // create offer
    const offer = await peerConnection.createOffer();
    await peerConnection.setLocalDescription(offer);

    client.watch(offer)
}

main()