use tokio_postgres::Client;

use object::interfaces::dealer::Dealer;

pub struct Service {
    pub dealer: Dealer,
    pub client: Client
}
