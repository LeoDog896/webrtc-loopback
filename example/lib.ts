export class Client {
  server: string;

  constructor(server: string) {
    this.server = server;
  }

  async watch(offer: RTCSessionDescriptionInit): Promise<RTCSessionDescriptionInit> {
    console.log(this.server + "/api/watch");
    const request = await fetch(this.server + "/api/watch", {
      method: "POST",
      body: JSON.stringify(offer),
    });

    const data = await request.text();

    try {
      const answer = JSON.parse(data);
      return answer as RTCSessionDescriptionInit;
    } catch (e) {
      throw new Error("An error occured: " + data);
    }
  }
}
