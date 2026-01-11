#![allow(missing_docs)]

use maud::{Markup, html};
use poem::{
    Response, Route, Server, error::BadRequest, handler, http::StatusCode, listener::TcpListener,
    web::Multipart,
};

use crate::render::render_to_texture;
pub mod render;

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    env_logger::init();

    let app = Route::new().at("/", poem::get(get).post(post));

    let port = std::env::var("PORT").unwrap_or("3000".to_owned());

    println!("Listening on https://localhost:{port}");
    Server::new(TcpListener::bind(format!("0.0.0.0:{port}")))
        .name("hello-world")
        .run(app)
        .await
}

#[handler]
pub async fn get() -> Markup {
    html! {
        form method="POST" enctype="multipart/form-data" {
            input type="file" name="file" accept=".charinfo";

            button { "Submit" }
        }
    }
}

#[handler]
pub async fn post(mut multipart: Option<Multipart>) -> poem::Result<Response> {
    let res_path = format!(
        "{}/resources_here",
        std::env::var("CARGO_WORKSPACE_DIR").unwrap()
    );

    // Optionally accept a file, otherwise use a default `.charinfo`.
    let mii_data = if let Some(mut multipart) = multipart {
        let mut mii_data = vec![];

        while let Some(field) = multipart.next_field().await? {
            let new_data = field.bytes().await.map_err(BadRequest)?;
            mii_data.extend(new_data);
        }

        mii_data
    } else {
        std::fs::read(format!("{res_path}/Jasmine.charinfo")).unwrap()
    };

    let image_buffer = render_to_texture(&mii_data, &res_path, 512, 512)
        .await
        .unwrap();

    let mut bytes = Vec::new();
    image_buffer
        .write_to(
            &mut std::io::Cursor::new(&mut bytes),
            image::ImageFormat::Png,
        )
        .map_err(|e| poem::Error::from_string(e.to_string(), StatusCode::INTERNAL_SERVER_ERROR))?;

    Ok(Response::builder().content_type("image/png").body(bytes))
}

#[cfg(test)]
mod test {
    use poem::{Route, test::TestClient};

    #[tokio::test]
    async fn quick_render() {
        let app = Route::new().at("/", poem::post(super::post));
        let cli = TestClient::new(app);

        // send request
        let resp = cli.post("/").send().await;
        // check the status code
        resp.assert_status_is_ok();
    }
}
