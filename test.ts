const user = { username: "test", password: "test" };

const register = await fetch("http://localhost:3030/register", {
  method: "POST",
  body: JSON.stringify(user),
  // content type json
  headers: {
    "Content-Type": "application/json",
  },
});

const registerResponse: {
  status: string;
  token: string;
} = await register.json();

// websocket client
const ws = new WebSocket(`ws://localhost:3030/game/${registerResponse.token}`);

ws.onopen = () => {
  console.log("connected");
};

ws.onmessage = (event) => {
  console.log(event.data);
};

ws.onclose = () => {
  console.log("disconnected");
}