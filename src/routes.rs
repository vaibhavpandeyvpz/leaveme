use crate::config;
use crate::slack::{
    add_reaction, get_message_link, send_leave_request, send_text_message, show_leave_form_view,
};
use rocket::form::Form;
use serde::Deserialize;
use serde_json::Value;
use std::collections::HashMap;

#[get("/")]
pub(crate) fn index() -> &'static str {
    "OK"
}

#[post("/slack/command", data = "<cmd>")]
pub(crate) async fn slack_command(cmd: Form<SlashCommand>) {
    if cmd.command == "/leave-me" {
        show_leave_form_view(cmd.channel_id.clone(), cmd.trigger_id.clone()).await;
    }
}

#[post("/slack/interaction", data = "<interaction>")]
pub(crate) async fn slack_interaction(interaction: Form<Interaction>) {
    let payload: InteractionPayload = serde_json::from_str(&interaction.payload).unwrap();
    if payload.r#type == "block_actions" && payload.actions.is_some() {
        let message = payload.message.as_ref().unwrap();
        let actions = payload.actions.as_ref().unwrap();
        for action in actions.iter() {
            if action.action_id == "approve_leave_request"
                || action.action_id == "reject_leave_request"
            {
                let value = action.value.as_ref().unwrap().as_str().unwrap();
                let user_from_until = value.split("|").collect::<Vec<&str>>();
                if action.action_id == "approve_leave_request" {
                    add_reaction(
                        config::<String>("slack.channels.leaves").into(),
                        message.ts.clone(),
                        "white_check_mark".into(),
                    )
                    .await;
                    send_text_message(
                        config::<String>("slack.channels.leaves").into(),
                        format!("<@{}> has *approved* this leave request.", payload.user.id).into(),
                        Some(message.ts.clone()),
                        None,
                    )
                    .await;
                    send_text_message(
                        user_from_until[0].into(),
                        format!(
                            "Your leave request from `{}` to `{}` has been approved :smile: by <@{}>.",
                            user_from_until[1],
                            user_from_until[2],
                            payload.user.id
                        ).into(),
                        None,
                        None
                    ).await;
                } else if action.action_id == "reject_leave_request" {
                    add_reaction(
                        config::<String>("slack.channels.leaves").into(),
                        message.ts.clone(),
                        "x".into(),
                    )
                    .await;
                    send_text_message(
                        config::<String>("slack.channels.leaves").into(),
                        format!("<@{}> has *rejected* this leave request.", payload.user.id).into(),
                        Some(message.ts.clone()),
                        None,
                    )
                    .await;
                    send_text_message(
                        user_from_until[0].into(),
                        format!(
                            "Your leave request from `{}` to `{}` has been rejected :sob: by <@{}>.",
                            user_from_until[1],
                            user_from_until[2],
                            payload.user.id
                        ).into(),
                        None,
                        None
                    ).await;
                }
            }
        }
    }

    if payload.r#type == "view_submission" && payload.view.is_some() {
        let view = payload.view.as_ref().unwrap();
        if view.callback_id == "submit_leave_request" {
            let values = &view.state.values;

            let from = &values["leave_request_from"]["leave_request_from_input"]["selected_date"];
            let until =
                &values["leave_request_until"]["leave_request_until_input"]["selected_date"];
            let reason = &values["leave_request_reason"]["leave_request_reason_input"]["value"];
            let reason_as_str: Option<String>;
            if reason.is_some() {
                reason_as_str = Some(reason.as_ref().unwrap().clone())
            } else {
                reason_as_str = None
            }

            let ts = send_leave_request(
                config::<String>("slack.channels.leaves").into(),
                payload.user.id.clone(),
                from.as_ref().unwrap().clone(),
                until.as_ref().unwrap().clone(),
                reason_as_str,
            )
            .await;
            let permalink =
                get_message_link(config::<String>("slack.channels.leaves").into(), ts).await;
            let managers = config::<Vec<String>>("managers");

            for manager in managers.iter() {
                send_text_message(
                    manager.into(),
                    format!("<@{}> has submitted a leave request. Please <{}|click here> to approve/reject.", payload.user.id, permalink).into(),
                    None,
                    None,
                ).await;
            }

            let _ = send_text_message(
                view.private_metadata.as_ref().unwrap().clone(),
                "Your request for leave has been submitted for approval.".into(),
                None,
                Some(payload.user.id.clone()),
            );
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
    values: HashMap<String, HashMap<String, HashMap<String, Option<String>>>>,
}

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