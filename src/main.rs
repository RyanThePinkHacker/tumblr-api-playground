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
    post::{
        ContentSubtype, Formatting, PostContent, PostCreate, PostCreateRequest, PostGetRequest,
        PostState, ReblogInfo,
    },
    TumblrClient,
};

use crate::posting::image_to_content;

const CLIENT_CACHE_PATH: &str = "client.json";
const BLOG_NAME: &str = "the-pink-hacker";
const STANDARD_TAGS: &str = "tumblr api,api,the pink hacker,tumblr api shenanigans";

fn yes_no(message: &str) {
    print!("{} [y/N] ", message);

    let mut should_post = String::new();
    stdout().flush().expect("Failed to flush stdout.");
    stdin()
        .read_line(&mut should_post)
        .expect("Failed to read line.");
    let should_post: &str = &should_post.to_lowercase();

    match should_post.trim() {
        "y" | "yes" => (),
        _ => panic!("Canceled"),
    }
}

#[allow(dead_code)]
async fn post_image_text(
    tumblr_client: &mut TumblrClient,
) -> Result<(), Box<dyn std::error::Error>> {
    let post_image = load_image::load_path("resources/infinite_8bit.png")?;

    let post = PostCreate {
        content: vec![image_to_content(post_image)?],
        state: Some(PostState::Draft),
        tags: Some("tumblr api,api,the pink hacker,tumblr api shenanigans,text art".to_string()),
        ..Default::default()
    };

    println!("{:#?}", post);

    yes_no("Send post?");

    let response = tumblr_client
        .send_request(&PostCreateRequest {
            blog_id: TumblrBlogId::BlogName(BLOG_NAME.to_string()),
            parameters: post,
        })
        .await?;
    println!("{:#?}", response);
    Ok(())
}

#[allow(dead_code)]
async fn post_reblog(tumblr_client: &mut TumblrClient) -> Result<(), Box<dyn std::error::Error>> {
    let response = tumblr_client
        .send_request(&PostGetRequest {
            blog_id: TumblrBlogId::BlogName("the-arcade-doctor".to_string()),
            post_id: "724283185473732608".to_string(),
        })
        .await?;
    println!("{:#?}", response);

    let parent_post_id = response.response.parameters.id;
    let reblog_key = response.response.parameters.reblog_key;
    let parent_tumblelog_uuid = response.response.parameters.tumblelog_uuid;
    println!(
        "parent_post_id: {}, reblog_key: {}, parent_tumblelog_uuid: {}",
        parent_post_id, reblog_key, parent_tumblelog_uuid
    );

    let post = PostCreate {
        content: vec![PostContent::Text {
            text: "[/81 Finite".to_string(),
            subtype: Some(ContentSubtype::Chat),
            indent_level: None,
            formatting: Some(vec![
                Formatting::Color {
                    start: 0,
                    end: 2,
                    hex: "#742372".to_string(),
                },
                Formatting::Color {
                    start: 2,
                    end: 11,
                    hex: "#db69d8".to_string(),
                },
                Formatting::Bold { start: 0, end: 11 },
            ]),
        }],
        state: Some(PostState::Draft),
        tags: Some(format!("{},i can now reblog with the api!", STANDARD_TAGS)),
        reblog_info: Some(ReblogInfo {
            parent_tumblelog_uuid,
            reblog_key,
            parent_post_id,
            ..Default::default()
        }),
        ..Default::default()
    };

    println!("{:#?}", post);

    yes_no("Send post?");

    let response = tumblr_client
        .send_request(&PostCreateRequest {
            blog_id: TumblrBlogId::BlogName(BLOG_NAME.to_string()),
            parameters: post,
        })
        .await?;
    println!("{:#?}", response);
    Ok(())
}

#[allow(dead_code)]
async fn post_video_text(
    tumblr_client: &mut TumblrClient,
) -> Result<(), Box<dyn std::error::Error>> {
    // Create first post
    let images_amount = 20;
    let post_image = load_image::load_path("resources/never_gonna_give_you_up/0.png")?;

    let post = PostCreate {
        content: vec![image_to_content(post_image)?],
        state: Some(PostState::Published),
        tags: Some(format!("{},text art,frame 0", STANDARD_TAGS)),
        ..Default::default()
    };

    println!("{:#?}", post);

    yes_no("Send post?");

    let response = tumblr_client
        .send_request(&PostCreateRequest {
            blog_id: TumblrBlogId::BlogName(BLOG_NAME.to_string()),
            parameters: post,
        })
        .await?;

    // Get info about first post
    let response = tumblr_client
        .send_request(&PostGetRequest {
            blog_id: TumblrBlogId::BlogName(BLOG_NAME.to_string()),
            post_id: response.response.id,
        })
        .await?
        .response
        .parameters;

    let mut parent_tumblelog_uuid = response.tumblelog_uuid;
    let mut parent_post_id = response.id;
    let mut reblog_key = response.reblog_key;

    // Reblogs
    for i in 1..images_amount {
        let post_image =
            load_image::load_path(format!("resources/never_gonna_give_you_up/{}.png", i))?;

        // Create reblog
        let response = tumblr_client
            .send_request(&PostCreateRequest {
                blog_id: TumblrBlogId::BlogName(BLOG_NAME.to_string()),
                parameters: PostCreate {
                    content: vec![image_to_content(post_image)?],
                    state: Some(PostState::Published),
                    tags: Some(format!("{},text art,frame {}", STANDARD_TAGS, i)),
                    reblog_info: Some(ReblogInfo {
                        parent_tumblelog_uuid: parent_tumblelog_uuid.clone(),
                        parent_post_id,
                        reblog_key: reblog_key.clone(),
                        ..Default::default()
                    }),
                    ..Default::default()
                },
            })
            .await?;

        // Get reblog info
        let response = tumblr_client
            .send_request(&PostGetRequest {
                blog_id: TumblrBlogId::BlogName(BLOG_NAME.to_string()),
                post_id: response.response.id.to_string(),
            })
            .await?
            .response
            .parameters;

        parent_tumblelog_uuid = response.tumblelog_uuid;
        parent_post_id = response.id;
        reblog_key = response.reblog_key;

        println!("Frame posted [{}/{}]", i + 1, images_amount);
    }

    println!("It's done (;");

    Ok(())
}

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

    // post_image_text(&mut tumblr_client).await?;
    // post_reblog(&mut tumblr_client).await?;
    post_video_text(&mut tumblr_client).await?;

    tumblr_client.save_to_file(CLIENT_CACHE_PATH.into())?;
    Ok(())
}
