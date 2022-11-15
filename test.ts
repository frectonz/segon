type User = {
  username: string;
  password: string;
};
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
} | {
  type: "GameEnd",
  score: number
} | {
  type: "GameStart"
};


async function test(user: User) {
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

  const label = user.username + " : "
  console.log(label, registerResponse);

  // websocket client
  const ws = new WebSocket(`ws://localhost:3030/game/${registerResponse.token}`);

  ws.onopen = () => {
    ws.send(JSON.stringify({ type: "Answer", answer_idx: "Three" }));
    console.log(label, "connected");
  }

  ws.onmessage = (event) => {
    const msg: Message = JSON.parse(event.data)

    if (msg.type === "TimeTillGame") {
      console.log(label, "Time till game", msg.time)
    } else if (msg.type === "Question") {
      console.log(label, msg.question);
      for (const option of msg.options) {
        console.log(label, option);
      }
      ws.send(JSON.stringify({ type: "Answer", answer_idx: "One" }));
    } else if (msg.type === "Answer") {
      console.log(label, "Answer status", msg.status);
      console.log(label, "Answer is option", msg.answer_idx);
    } else if (msg.type === "NoGame") {
      console.log(label, "No game");
    } else if (msg.type === "GameEnd") {
      console.log(label, "Game ended with score", msg.score);
    } else if (msg.type === "GameStart") {
      console.log(label, "Game started");
    }

  };

  ws.onclose = () => {
    console.log(label, "disconnected");
  }
}

for (let i = 0; i < 10; i++) {
  test({ username: `user-${i}`, password: "password" });
}