use crate::{
    controllers::GameController,
    ports::{ClientsManager, GameDatabase, GameStartNotifier, JobSchedular},
};
use warp::ws::WebSocket;

pub async fn connect<
    GD: GameDatabase + Send + Sync + Clone + 'static,
    CM: ClientsManager + Send + Sync + Clone + 'static,
    JS: JobSchedular + Send + Sync + Clone + 'static,
    GSN: GameStartNotifier + Send + Sync + Clone + 'static,
>(
    controller: GameController<GD, CM, JS, GSN>,
    ws: WebSocket,
) {
    controller.start(ws).await;
    println!("disconnected");
}

// add them to a list of connected user -> done
// tell the client how much time is left until the game starts -> done
// when the timer reaches the game time send a start signal and the first question to the client
// receive answers for 10 seconds until the question timer ends
// send the answer to all of the connected clients and whether or not they were correct
// drop off the clients that have answered wrong from the answerers list
// repeat this process until all questions have been answered
// assemble a leaderboard and send it to every client with each client's rank on the leaderboard
