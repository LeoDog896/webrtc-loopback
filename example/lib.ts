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
        
        const data = await request.text()

        try {
            const answer = JSON.parse(data)
            return answer;
        } catch (e) {
            throw new Error("An error occured: " + data)
        }
    }
}