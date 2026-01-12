#![allow(missing_docs)]

use crate::render::render_to_texture;
use maud::{DOCTYPE, Markup, html};
use poem::{
    Response, Route, Server,
    error::BadRequest,
    handler,
    http::StatusCode,
    listener::TcpListener,
    web::{Form, Multipart},
};
use std::{collections::HashMap, io::Cursor, path::PathBuf};
use vfl::parse::{
    BinRead as _, CtrStoreData, NxCharInfo, RvlCharData, StudioCharInfo,
    generic::{AsGenericChar, FromGenericChar},
    studio::studio_url_obfuscation_decode,
};

pub mod render;

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    env_logger::init();

    let app = Route::new()
        .at("/", poem::get(get))
        .at("/charinfo", poem::post(charinfo))
        .at("/ctr", poem::post(ctr))
        .at("/rvl_char", poem::post(rvl_char))
        .at("/studio", poem::post(studio));

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
        ( DOCTYPE )
        html {
            head {
                title { "VEE Icon Renderer" }
                meta name="viewport" content="width=device-width";
                meta name="color-scheme" content="light dark";
            }
        }

        body {

            h1 { "Vee Icon Renderer (Alpha)" }

            p {
                "This is a work-in-progress renderer. Known missing features:"
            }
            ul {
                li { "Shading" }
                li { "Camera positioning and clear color changing" }
                li { "Body scaling" }
            }
            a href="https://github.com/j0lol/vee/tree/main/crates/render_server" { "View source" }

            hr;

            form method="POST" action="/charinfo" enctype="multipart/form-data" {
                label {
                    "NxCharInfo, nn::mii::CharInfo"
                    br;
                    input type="file" name="file" accept=".charinfo";
                }
                br;
                button { "Submit NxCharInfo" }
            }
            hr;
            form method="POST" action="/ctr" enctype="multipart/form-data" {
                label {
                    "Ver3StoreData, FFSD, CFSD"
                    br;
                    input type="file" name="file" accept=".ctrstoredata,.ffsd";
                }
                br;
                button { "Submit CtrStoreData" }
            }
            hr;
            form method="POST" action="/rvl_char" enctype="multipart/form-data" {
                label {
                    "RvlCharData, \"RFLCharData\""
                    br;
                    input type="file" name="file" accept=".rvlchardata,.rcd";
                }
                br;
                button { "Submit RvlCharData" }
            }
            hr;
            form method="POST" action="/studio" {
                label {
                    "Hex-encoded (base16) CharInfo from Mii Studio"
                    br;
                    input type="text" name="data" required size="47" pattern="[a-fA-F0-9]{92,94}";
                }
                br;
                button { "Submit StudioCharInfo" }
            }
        }

    }
}

async fn render_charinfo(char_info: NxCharInfo) -> poem::Result<Response> {
    let res_path: PathBuf = [
        std::env::var("CARGO_WORKSPACE_DIR").unwrap(),
        "resources_here".to_string(),
    ]
    .iter()
    .collect();

    let image_buffer = render_to_texture(&char_info, &res_path, 512, 512)
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

#[handler]
pub async fn studio(mut form: Form<HashMap<String, String>>) -> poem::Result<Response> {
    let mii_data = {
        let data = form.get("data").unwrap();
        println!("Studio: [{data}];");

        // hex decode
        let mut data = hex::decode(data).unwrap();
        dbg!(data.len());

        let obfuscated = match data.len() {
            47 => true,
            46 => false,
            _ => panic!("This is not a studio mii."),
        };

        // deobfuscate
        if obfuscated {
            studio_url_obfuscation_decode(&mut data[..]);
        }

        data
    };

    // let char_info = StudioCharInfo::read(&mut Cursor::new(mii_data))
    //     .map_err(BadRequest)?
    //     .to_nxcharinfo();

    let char_data = StudioCharInfo::read(&mut Cursor::new(mii_data)).map_err(BadRequest)?;
    let char_info = NxCharInfo::from_generic(char_data.as_generic().unwrap());

    render_charinfo(char_info).await
}

#[handler]
pub async fn ctr(mut multipart: Multipart) -> poem::Result<Response> {
    let mut mii_data = vec![];

    while let Some(field) = multipart.next_field().await? {
        let new_data = field.bytes().await.map_err(BadRequest)?;
        mii_data.extend(new_data);
    }

    let ctr_store_data = CtrStoreData::read(&mut Cursor::new(mii_data)).map_err(BadRequest)?;
    let char_info = NxCharInfo::from_generic(ctr_store_data.as_generic().unwrap());

    render_charinfo(char_info).await
}

#[handler]
pub async fn rvl_char(mut multipart: Multipart) -> poem::Result<Response> {
    let mut mii_data = vec![];

    while let Some(field) = multipart.next_field().await? {
        let new_data = field.bytes().await.map_err(BadRequest)?;
        mii_data.extend(new_data);
    }

    let char_data = RvlCharData::read(&mut Cursor::new(mii_data)).map_err(BadRequest)?;
    let char_info = NxCharInfo::from_generic(char_data.as_generic().unwrap());

    render_charinfo(char_info).await
}

#[handler]
pub async fn charinfo(mut multipart: Option<Multipart>) -> poem::Result<Response> {
    let res_path: PathBuf = [
        std::env::var("CARGO_WORKSPACE_DIR").unwrap(),
        "resources_here".to_string(),
    ]
    .iter()
    .collect();

    // Optionally accept a file, otherwise use a default `.charinfo`.
    let mii_data = if let Some(mut multipart) = multipart {
        let mut mii_data = vec![];

        while let Some(field) = multipart.next_field().await? {
            let new_data = field.bytes().await.map_err(BadRequest)?;
            mii_data.extend(new_data);
        }

        mii_data
    } else {
        std::fs::read(res_path.join("Jasmine.charinfo")).unwrap()
    };

    let char_info = NxCharInfo::read(&mut Cursor::new(mii_data)).map_err(BadRequest)?;

    render_charinfo(char_info).await
}

#[cfg(test)]
mod test {
    use poem::{Route, test::TestClient};

    #[tokio::test]
    async fn quick_render() {
        let app = Route::new().at("/", poem::post(super::charinfo));
        let cli = TestClient::new(app);

        // send request
        let resp = cli.post("/").send().await;
        // check the status code
        resp.assert_status_is_ok();
    }
}
