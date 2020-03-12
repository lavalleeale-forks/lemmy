pub mod community;
pub mod post;
pub mod puller;
pub mod user;
use crate::Settings;
use failure::Error;

use std::fmt::Display;
use url::Url;

#[cfg(test)]
mod tests {
  use crate::db::community::Community;
  use crate::db::post::Post;
  use crate::db::user::User_;
  use crate::db::{ListingType, SortType};
  use crate::{naive_now, Settings};

  #[test]
  fn test_person() {
    let user = User_ {
      id: 52,
      name: "thom".into(),
      fedi_name: "rrf".into(),
      preferred_username: None,
      password_encrypted: "here".into(),
      email: None,
      matrix_user_id: None,
      avatar: None,
      published: naive_now(),
      admin: false,
      banned: false,
      updated: None,
      show_nsfw: false,
      theme: "darkly".into(),
      default_sort_type: SortType::Hot as i16,
      default_listing_type: ListingType::Subscribed as i16,
      lang: "browser".into(),
      show_avatars: true,
      send_notifications_to_email: false,
    };

    let person = user.as_person();
    assert_eq!(
      format!("https://{}/federation/u/thom", Settings::get().hostname),
      person.unwrap().object_props.get_id().unwrap().to_string()
    );
  }

  #[test]
  fn test_community() {
    let community = Community {
      id: 42,
      name: "Test".into(),
      title: "Test Title".into(),
      description: Some("Test community".into()),
      category_id: 32,
      creator_id: 52,
      removed: false,
      published: naive_now(),
      updated: Some(naive_now()),
      deleted: false,
      nsfw: false,
    };

    let group = community.as_group();
    assert_eq!(
      format!("https://{}/federation/c/Test", Settings::get().hostname),
      group.unwrap().object_props.get_id().unwrap().to_string()
    );
  }

  #[test]
  fn test_post() {
    let post = Post {
      id: 62,
      name: "A test post".into(),
      url: None,
      body: None,
      creator_id: 52,
      community_id: 42,
      published: naive_now(),
      removed: false,
      locked: false,
      stickied: false,
      nsfw: false,
      deleted: false,
      updated: None,
    };

    let page = post.as_page();
    assert_eq!(
      format!("https://{}/federation/post/62", Settings::get().hostname),
      page.unwrap().object_props.get_id().unwrap().to_string()
    );
  }
}

// TODO: this should take an enum community/user/post for `point`
// TODO: also not sure what exactly `value` should be (numeric id, name string, ...)
pub fn make_apub_endpoint<S: Display, T: Display>(point: S, value: T) -> Url {
  Url::parse(&format!(
    "{}://{}/federation/{}/{}",
    get_apub_protocol_string(),
    Settings::get().hostname,
    point,
    value
  ))
  .unwrap()
}

/// Parses an ID generated by `make_apub_endpoint()`. Will break when federating with anything
/// that is not Lemmy. This is just a crutch until we change the database to store URLs as ID.
pub fn parse_apub_endpoint(id: &str) -> Result<(&str, &str), Error> {
  let split = id.split('/').collect::<Vec<&str>>();
  Ok((split[4], split[5]))
}

pub fn get_apub_protocol_string() -> &'static str {
  "http"
}
