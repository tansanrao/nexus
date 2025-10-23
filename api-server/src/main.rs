#[rocket::launch]
fn rocket() -> _ {
    api_server::rocket()
}
