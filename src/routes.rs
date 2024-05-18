use crate::config;
use crate::slack::{
    show_leave_form_view, submit_leave_request, update_leave_request, /*, verify_request */
};
use rocket::form::Form;
use rocket::http::Status;
use rocket::request::{FromRequest, Outcome, Request};
use serde::Deserialize;
use serde_json::Value;
use std::collections::HashMap;
use url_encoded_data::UrlEncodedData;

#[get("/")]
pub(crate) fn index() -> &'static str {
    "OK"
}

#[post("/slack/command", data = "<cmd>")]
pub(crate) async fn slack_command(
    cmd: Form<SlashCommand>,
    _timestamp: XSlackRequestTimestamp<'_>,
    _signature: XSlackSignature<'_>,
) -> Status {
    // let data = String::new();
    // verify_request(
    //     &data,
    //     &config::<String>("slack.signing_secret"),
    //     &timestamp.0.to_string(),
    //     &signature.0.to_string(),
    // ).await;

    return if cmd.command == "/leave-me" {
        show_leave_form_view(&cmd.channel_id, &cmd.trigger_id).await;

        Status::Ok
    } else {
        Status::NotFound
    };
}

#[post("/slack/interaction", data = "<interaction>")]
pub(crate) async fn slack_interaction(
    interaction: Form<Interaction>,
    _timestamp: XSlackRequestTimestamp<'_>,
    _signature: XSlackSignature<'_>,
) -> Status {
    // let data = String::new();
    // verify_request(
    //     &data,
    //     &config::<String>("slack.signing_secret"),
    //     &timestamp.0.to_string(),
    //     &signature.0.to_string(),
    // ).await;

    let payload: InteractionPayload = serde_json::from_str(&interaction.payload).unwrap();
    if payload.r#type == "block_actions" && payload.actions.is_some() {
        let message = payload.message.unwrap();
        let actions = payload.actions.unwrap();
        for action in actions.iter() {
            if action.action_id == "approve_leave_request"
                || action.action_id == "reject_leave_request"
            {
                let value = action.value.as_ref().unwrap().as_str().unwrap();
                let decoded = UrlEncodedData::from(value);
                let user = decoded.get_first("user").unwrap();
                let from = decoded.get_first("from").unwrap();
                let until = decoded.get_first("until").unwrap();
                let reason = decoded.get_first("reason").unwrap();

                update_leave_request(
                    &config::<String>("slack.channels.leaves"),
                    &user.to_string(),
                    &from.to_string(),
                    &until.to_string(),
                    &reason.to_string(),
                    &payload.user.id.to_string(),
                    action.action_id == "approve_leave_request",
                    &message.ts,
                )
                .await;

                return Status::Ok;
            }
        }
    } else if payload.r#type == "view_submission" && payload.view.is_some() {
        let view = payload.view.as_ref().unwrap();
        if view.callback_id == "submit_leave_request" {
            let values = &view.state.values;
            submit_leave_request(
                view.private_metadata.as_ref().clone().unwrap(),
                &config::<String>("slack.channels.leaves"),
                &payload.user.id,
                &values,
            )
            .await;

            return Status::Ok;
        }
    }

    return Status::NotFound;
}

#[derive(FromForm)]
pub(crate) struct SlashCommand {
    pub(crate) command: String,
    pub(crate) trigger_id: String,
    pub(crate) channel_id: String,
}

#[derive(FromForm)]
pub(crate) struct Interaction {
    payload: String,
}

#[derive(Deserialize)]
pub(crate) struct InteractionAction {
    action_id: String,
    value: Option<Value>,
}

#[derive(Deserialize)]
pub(crate) struct InteractionMessage {
    ts: String,
}

#[derive(Deserialize)]
pub(crate) struct InteractionPayload {
    r#type: String,
    user: InteractionUser,
    message: Option<InteractionMessage>,
    view: Option<InteractionView>,
    actions: Option<Vec<InteractionAction>>,
}

#[derive(Deserialize)]
pub(crate) struct InteractionState {
    values: InteractionStateValues,
}

pub(crate) type InteractionStateValues =
    HashMap<String, HashMap<String, HashMap<String, Option<String>>>>;

#[derive(Deserialize)]
pub(crate) struct InteractionUser {
    id: String,
}

#[derive(Deserialize)]
pub(crate) struct InteractionView {
    callback_id: String,
    private_metadata: Option<String>,
    state: InteractionState,
}

pub(crate) struct XSlackRequestTimestamp<'r>(&'r str);
pub(crate) struct XSlackSignature<'r>(&'r str);

#[rocket::async_trait]
impl<'r> FromRequest<'r> for XSlackRequestTimestamp<'r> {
    type Error = ();

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let timestamp = request.headers().get_one("x-slack-request-timestamp");
        match timestamp {
            Some(timestamp) => Outcome::Success(XSlackRequestTimestamp(timestamp)),
            None => Outcome::Error((Status::Unauthorized, ())),
        }
    }
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for XSlackSignature<'r> {
    type Error = ();

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let signature = request.headers().get_one("x-slack-signature");
        match signature {
            Some(signature) => Outcome::Success(XSlackSignature(signature)),
            None => Outcome::Error((Status::Unauthorized, ())),
        }
    }
}
