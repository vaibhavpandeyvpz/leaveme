use crate::config;
use crate::slack::{show_leave_form_view, submit_leave_request, update_leave_request};
use rocket::form::Form;
use serde::Deserialize;
use serde_json::Value;
use std::collections::HashMap;
use url_encoded_data::UrlEncodedData;

#[get("/")]
pub(crate) fn index() -> &'static str {
    "OK"
}

#[post("/slack/command", data = "<cmd>")]
pub(crate) async fn slack_command(cmd: Form<SlashCommand>) {
    if cmd.command == "/leave-me" {
        show_leave_form_view(&cmd.channel_id, &cmd.trigger_id).await;
    }
}

#[post("/slack/interaction", data = "<interaction>")]
pub(crate) async fn slack_interaction(interaction: Form<Interaction>) {
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
            }
        }
    }

    if payload.r#type == "view_submission" && payload.view.is_some() {
        let view = payload.view.as_ref().unwrap();
        if view.callback_id == "submit_leave_request" {
            let values = &view.state.values;
            submit_leave_request(
                view.private_metadata.as_ref().clone().unwrap(),
                &config::<String>("slack.channels.leaves"),
                &payload.user.id,
                &values,
            )
            .await
        }
    }
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
