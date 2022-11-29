import "./index.css";
import { Elm } from "./Main.elm";
import confetti from "canvas-confetti";

const app = Elm.Main.init({
  node: document.getElementById("app"),
});

let ws;
app.ports.connectToGameServer.subscribe((token) => {
  ws = new WebSocket(`ws://localhost:3030/game/${token}`);
  ws.addEventListener("open", () => {
    console.log("Connected to game server", token);
  });

  ws.addEventListener("message", (event) => {
    const data = JSON.parse(event.data);
    app.ports.receiveGameServerMessage.send(data);
  });
});

app.ports.sendGameServerMessage.subscribe((data) => {
  ws.send(JSON.stringify(data));
});

app.ports.confetti.subscribe((data) => {
  confetti();
});
