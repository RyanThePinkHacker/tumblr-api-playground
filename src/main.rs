mod posting {
    use load_image::{Image, ImageData};
    use tumblr_api::post::{Formatting, PostContent};

    const SQUARE: char = '■';

    pub fn image_to_content(image: Image) -> Result<PostContent, Box<dyn std::error::Error>> {
        if let ImageData::RGB16(color) = image.bitmap {
            let mut text = String::with_capacity(image.width);
            let mut formatting = Vec::new();

            for height in 0..image.height {
                for width in 0..image.width {
                    let pixel_width = 2;
                    let index: usize = (height * image.width) + width;
                    let index_new_line_corrected =
                        (height * (image.width * pixel_width)) + (width * pixel_width) + height;
                    text.push(SQUARE);
                    text.push(' ');

                    let color = color.get(index).ok_or("Out of bounds.")?;

                    let (r, g, b) = (
                        hex_string::u8_to_hex_string(&((color.r / 256) as u8))
                            .iter()
                            .collect::<String>(),
                        hex_string::u8_to_hex_string(&((color.g / 256) as u8))
                            .iter()
                            .collect::<String>(),
                        hex_string::u8_to_hex_string(&((color.b / 256) as u8))
                            .iter()
                            .collect::<String>(),
                    );

                    formatting.push(Formatting::Color {
                        start: index_new_line_corrected as u32,
                        end: (index_new_line_corrected + pixel_width) as u32,
                        hex: format!("#{}{}{}", r, g, b),
                    });
                }
                text.push('\n');
            }

            Ok(PostContent::Text {
                text,
                subtype: None,
                indent_level: None,
                formatting: Some(formatting),
            })
        } else {
            Err("Incorrect color format.")?
        }
    }
}

use std::io::{stdin, stdout, Write};

use tumblr_api::{
    auth::read_credentials,
    blog::TumblrBlogId,
    post::{Post, PostCreateRequest, PostCreateResponse, PostState},
    requests::TumblrResponse,
    TumblrClient,
};

use crate::posting::image_to_content;

const CLIENT_CACHE_PATH: &str = "client.json";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let credentials = read_credentials()?;
    let mut tumblr_client = TumblrClient::try_from_file_or_authorize(
        CLIENT_CACHE_PATH.into(),
        credentials,
        reqwest::Client::new(),
    )
    .await?;
    tumblr_client.save_to_file(CLIENT_CACHE_PATH.into())?;

    let post_image = load_image::load_path("resources/infinite_8bit.png")?;

    let post = Post {
        content: vec![image_to_content(post_image)?],
        state: Some(PostState::Draft),
        tags: Some("tumblr api,api,the pink hacker,tumblr api shenanigans,text art".to_string()),
        ..Default::default()
    };

    print!("Post? [y/N] ");

    let mut should_post = String::new();
    stdout().flush()?;
    stdin().read_line(&mut should_post)?;
    let should_post: &str = &should_post.to_lowercase();

    match should_post.trim() {
        "y" | "yes" => (),
        _ => return Ok(()),
    }

    println!("{:#?}", post);

    let response = tumblr_client
        .request::<TumblrResponse<PostCreateResponse>>(
            PostCreateRequest {
                blog_id: TumblrBlogId::BlogName("the-pink-hacker".to_string()),
                parameters: post,
            }
            .try_into()?,
        )
        .await?;
    println!("{:#?}", response);
    Ok(())
}
