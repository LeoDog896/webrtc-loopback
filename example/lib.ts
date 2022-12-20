export class Client {

    server: string;

    constructor(server: string) {
        this.server = server;
    }

    async watch(offer: RTCSessionDescriptionInit) {
        const request = await fetch(this.server + "/api/watch", {
            method: "POST",
            body: JSON.stringify(offer)
        })
        const answer = await request.text()
        console.log(answer)
    }
}