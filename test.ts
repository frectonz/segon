const user = { username: "test", password: "test" };

const register = await fetch("http://localhost:3030/register", {
  method: "POST",
  body: JSON.stringify(user),
  headers: {
    "Content-Type": "application/json",
  },
});

const registerResponse: {
  status: string;
  token: string;
} = await register.json();

console.log("RESPONSE", registerResponse);

// websocket client
const ws = new WebSocket(`ws://localhost:3030/game/${registerResponse.token}`);

ws.onopen = () => {
  ws.send(JSON.stringify({ type: "Answer", answer_idx: "Three" }));
  console.log("connected");
}

type Message = {
  type: "TimeTillGame",
  time: number
} | {
  type: "Question",
  question: string,
  options: string[]
} | {
  type: "Answer",
  status: string,
  answer_idx: "One" | "Two" | "Three" | "Four"
} | {
  type: "NoGame"
}

ws.onmessage = (event) => {
  const msg: Message = JSON.parse(event.data)

  if (msg.type === "TimeTillGame") {
    console.log("Time till game", msg.time)
  } else if (msg.type === "Question") {
    console.log(msg.question);
    for (const [idx, option] of msg.options.entries()) {
      console.log("  ", idx + 1, option);
    }
    ws.send(JSON.stringify({ type: "Answer", answer_idx: "Three" }));
  } else if (msg.type === "Answer") {
    console.log("Answer status", msg.status);
    console.log("Answer is option", msg.answer_idx);
  } else if (msg.type === "NoGame") {
    console.log("No game");
  }

};

ws.onclose = () => {
  console.log("disconnected");
}