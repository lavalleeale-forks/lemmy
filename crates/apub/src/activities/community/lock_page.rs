use crate::{
  activities::{
    check_community_deleted_or_removed,
    community::send_activity_in_community,
    generate_activity_id,
    verify_is_public,
    verify_mod_action,
    verify_person_in_community,
  },
  activity_lists::AnnouncableActivities,
  insert_received_activity,
  objects::community::ApubCommunity,
  protocol::{
    activities::community::lock_page::{LockPage, LockType, UndoLockPage},
    InCommunity,
  },
};
use activitypub_federation::{
  config::Data,
  fetch::object_id::ObjectId,
  kinds::{activity::UndoType, public},
  traits::ActivityHandler,
};
use lemmy_api_common::context::LemmyContext;
use lemmy_db_schema::{
  source::{
    community::Community,
    person::Person,
    post::{Post, PostUpdateForm},
  },
  traits::Crud,
};
use lemmy_utils::error::LemmyError;
use url::Url;

#[async_trait::async_trait]
impl ActivityHandler for LockPage {
  type DataType = LemmyContext;
  type Error = LemmyError;

  fn id(&self) -> &Url {
    &self.id
  }

  fn actor(&self) -> &Url {
    self.actor.inner()
  }

  async fn verify(&self, context: &Data<Self::DataType>) -> Result<(), Self::Error> {
    verify_is_public(&self.to, &self.cc)?;
    let community = self.community(context).await?;
    verify_person_in_community(&self.actor, &community, context).await?;
    check_community_deleted_or_removed(&community)?;
    verify_mod_action(&self.actor, self.object.inner(), community.id, context).await?;
    Ok(())
  }

  async fn receive(self, context: &Data<Self::DataType>) -> Result<(), Self::Error> {
    let form = PostUpdateForm::builder().locked(Some(true)).build();
    let post = self.object.dereference(context).await?;
    Post::update(&mut context.pool(), post.id, &form).await?;
    Ok(())
  }
}

#[async_trait::async_trait]
impl ActivityHandler for UndoLockPage {
  type DataType = LemmyContext;
  type Error = LemmyError;

  fn id(&self) -> &Url {
    &self.id
  }

  fn actor(&self) -> &Url {
    self.actor.inner()
  }

  async fn verify(&self, context: &Data<Self::DataType>) -> Result<(), Self::Error> {
    insert_received_activity(&self.id, context).await?;
    verify_is_public(&self.to, &self.cc)?;
    let community = self.community(context).await?;
    verify_person_in_community(&self.actor, &community, context).await?;
    check_community_deleted_or_removed(&community)?;
    verify_mod_action(
      &self.actor,
      self.object.object.inner(),
      community.id,
      context,
    )
    .await?;
    Ok(())
  }

  async fn receive(self, context: &Data<Self::DataType>) -> Result<(), Self::Error> {
    let form = PostUpdateForm::builder().locked(Some(false)).build();
    let post = self.object.object.dereference(context).await?;
    Post::update(&mut context.pool(), post.id, &form).await?;
    Ok(())
  }
}

pub(crate) async fn send_lock_post(
  post: Post,
  actor: Person,
  locked: bool,
  context: Data<LemmyContext>,
) -> Result<(), LemmyError> {
  let community: ApubCommunity = Community::read(&mut context.pool(), post.community_id)
    .await?
    .into();
  let id = generate_activity_id(
    LockType::Lock,
    &context.settings().get_protocol_and_hostname(),
  )?;
  let community_id = community.actor_id.inner().clone();
  let lock = LockPage {
    actor: actor.actor_id.clone().into(),
    to: vec![public()],
    object: ObjectId::from(post.ap_id),
    cc: vec![community_id.clone()],
    kind: LockType::Lock,
    id,
    audience: Some(community_id.into()),
  };
  let activity = if locked {
    AnnouncableActivities::LockPost(lock)
  } else {
    let id = generate_activity_id(
      UndoType::Undo,
      &context.settings().get_protocol_and_hostname(),
    )?;
    let undo = UndoLockPage {
      actor: lock.actor.clone(),
      to: vec![public()],
      cc: lock.cc.clone(),
      kind: UndoType::Undo,
      id,
      audience: lock.audience.clone(),
      object: lock,
    };
    AnnouncableActivities::UndoLockPost(undo)
  };
  send_activity_in_community(activity, &actor.into(), &community, vec![], true, &context).await?;
  Ok(())
}
