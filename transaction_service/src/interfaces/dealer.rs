use tokio_postgres::Client;

use object::interfaces::dealer::Dealer;

pub struct DealerService {
    pub dealer: Dealer,
    pub client: Client
}