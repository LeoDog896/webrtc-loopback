import { Client } from "./lib"

const messageTextBox = document.querySelector<HTMLInputElement>("message")!;

const config = { iceServers: [{ urls: "stun:stun.l.google.com:19302" }] };

// The local browser's RTCPeerConnection
const peerConnection = new RTCPeerConnection(config);

const video = document.querySelector<HTMLVideoElement>("localVideo")!;

async function main() {
    const client = new Client("http://localhost:8080")

    const dataChannel = peerConnection.createDataChannel('dummy');

    // create offer
    await peerConnection.setLocalDescription(await peerConnection.createOffer());

    peerConnection.addEventListener("icecandidate", async ({ candidate }) => {
        if (candidate == null && peerConnection.localDescription != null) {
            const answer = await client.watch(peerConnection.localDescription);
            console.log(answer)
        } else if (candidate == null) {
            throw new Error("Local description is null! This should not happen.");
        }
    })
}

main()