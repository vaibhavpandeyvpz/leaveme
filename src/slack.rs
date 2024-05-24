use crate::config;
use crate::routes::InteractionStateValues;
use chrono::prelude::*;
use clap::builder::Str;
use config::Map;
use slack_http_verifier::SlackVerifier;
use slack_morphism::prelude::*;
use url_encoded_data::UrlEncodedData;

lazy_static::lazy_static! {
    static ref SLACK_CLIENT: SlackClient<SlackClientHyperHttpsConnector> =
        SlackClient::new(SlackClientHyperHttpsConnector::new().unwrap());

    static ref SLACK_TOKEN: SlackApiToken = SlackApiToken::new(
        SlackApiTokenValue::new(config::<String>("slack.bot_token"))
    );
}

pub(crate) async fn add_reaction(channel: &String, ts: &String, reaction: &String) {
    let reaction_add_req = SlackApiReactionsAddRequest::new(
        SlackChannelId::new(channel.into()),
        SlackReactionName::new(reaction.into()),
        ts.into(),
    );

    let _ = SLACK_CLIENT
        .open_session(&SLACK_TOKEN)
        .reactions_add(&reaction_add_req)
        .await;
}

pub(crate) async fn get_message_link(channel: &String, ts: &String) -> String {
    let chat_permalink_req =
        SlackApiChatGetPermalinkRequest::new(SlackChannelId::new(channel.into()), ts.into());
    let chat_permalink_resp = SLACK_CLIENT
        .open_session(&SLACK_TOKEN)
        .chat_get_permalink(&chat_permalink_req)
        .await
        .unwrap();

    chat_permalink_resp.permalink.to_string()
}

pub(crate) async fn send_leave_request(
    channel: &String,
    user: &String,
    from: &String,
    until: &String,
    full_or_half: &String,
    reason: Option<&String>,
) -> String {
    let from_dt = format!("{}T00:00:00Z", from)
        .parse::<DateTime<Utc>>()
        .unwrap();
    let from_dt_str = from_dt.format("%a, %b %e %Y").to_string();
    let until_dt = format!("{}T23:59:59Z", until)
        .parse::<DateTime<Utc>>()
        .unwrap();
    let until_dt_str = until_dt.format("%a, %b %e %Y").to_string();

    let full_or_half_str = match full_or_half.as_str() {
        "half" => "Half day".to_string(),
        _ => "Full day".to_string(),
    };
    let reason_str = match reason.is_some() {
        true => reason.unwrap().into(),
        false => "No reason provided.".to_string(),
    };
    let encoded = UrlEncodedData::parse_str("")
        .set_one("user", user)
        .set_one("from", &from_dt_str)
        .set_one("until", &until_dt_str)
        .set_one("full_or_half", &full_or_half_str)
        .set_one("reason", &reason_str)
        .done()
        .to_final_string();
    let blocks: Vec<SlackBlock> = slack_blocks![
        some_into(SlackSectionBlock::new().with_text(md!(format!(
            "<@{}> has submitted a request for leave from *{}* to *{}*.",
            user, from_dt_str, until_dt_str
        )))),
        some_into(SlackSectionBlock::new().with_text(md!(format!("*Full or half day:* {}", full_or_half_str)))),
        some_into(SlackSectionBlock::new().with_text(md!(format!("*Reason:* _{}_", reason_str)))),
        some_into(SlackActionsBlock::new(slack_blocks![
            some_into(
                SlackBlockButtonElement::new("approve_leave_request".into(), pt!("Approve"),)
                    .with_confirm(SlackBlockConfirmItem::new(
                        pt!("Really?"),
                        pt!("This will mark this request for leave as approved. It cannot be undone."),
                        pt!("Approve"),
                        pt!("Cancel")
                    ))
                    .with_value(encoded.clone())
                    .with_style("primary".into())
            ),
            some_into(
                SlackBlockButtonElement::new("reject_leave_request".into(), pt!("Reject"),)
                    .with_confirm(SlackBlockConfirmItem::new(
                        pt!("Really?"),
                        pt!("This will mark this request for leave as rejected. It cannot be undone."),
                        pt!("Reject"),
                        pt!("Cancel")
                    ))
                    .with_value(encoded)
                    .with_style("danger".into())
            )
        ],))
    ];

    let post_chat_req = SlackApiChatPostMessageRequest::new(
        SlackChannelId::new(channel.into()),
        SlackMessageContent::new().with_blocks(blocks),
    );
    let post_chat_resp = SLACK_CLIENT
        .open_session(&SLACK_TOKEN)
        .chat_post_message(&post_chat_req)
        .await
        .unwrap();

    post_chat_resp.ts.to_string()
}

pub(crate) async fn send_text_message(
    channel: &String,
    msg: &String,
    ts: Option<&String>,
    user: Option<&String>,
) -> Option<String> {
    return if user.is_some() {
        let post_ephemeral_req = SlackApiChatPostEphemeralRequest::new(
            channel.into(),
            SlackUserId::new(user.unwrap().into()),
            SlackMessageContent::new().with_text(msg.into()),
        );

        let _ = SLACK_CLIENT
            .open_session(&SLACK_TOKEN)
            .chat_post_ephemeral(&post_ephemeral_req)
            .await;

        None
    } else {
        let mut post_chat_req = SlackApiChatPostMessageRequest::new(
            channel.into(),
            SlackMessageContent::new().with_text(msg.into()),
        )
        .with_unfurl_links(false);

        if let Some(thread_ts) = ts {
            post_chat_req = post_chat_req.with_thread_ts(SlackTs::new(thread_ts.into()));
        }

        let post_chat_resp = SLACK_CLIENT
            .open_session(&SLACK_TOKEN)
            .chat_post_message(&post_chat_req)
            .await
            .unwrap();

        Some(post_chat_resp.ts.to_string())
    };
}

pub(crate) async fn show_leave_form_view(channel: &String, trigger_id: &String) {
    let blocks: Vec<SlackBlock> = slack_blocks![
        some_into(
            SlackSectionBlock::new()
                .with_text(pt!("Fill and submit below information to request a leave. Please be noted, the dates are inclusive."))
        ),
        some_into(
            SlackInputBlock::new(
                pt!("From date"),
                SlackBlockDatePickerElement::new("leave_request_from_input".into()).into()
            )
            .with_block_id("leave_request_from".into())
        ),
        some_into(
            SlackInputBlock::new(
                pt!("Last date"),
                SlackBlockDatePickerElement::new("leave_request_until_input".into()).into()
            )
            .with_block_id("leave_request_until".into())
        ),
        some_into(
            SlackInputBlock::new(
                pt!("Full or half-day"),
                SlackBlockRadioButtonsElement::new(
                    "leave_request_full_half_input".into(),
                    vec![
                        SlackBlockChoiceItem::new(pt!("Full day(s)"), "full".into()),
                        SlackBlockChoiceItem::new(pt!("Half day"), "half".into())
                    ]
                )
                .with_initial_option(
                    SlackBlockChoiceItem::new(pt!("Full day(s)"), "full".into()),
                )
                .into()
            )
            .with_block_id("leave_request_full_half".into())
        ),
        some_into(
            SlackInputBlock::new(
                pt!("Reason"),
                SlackBlockPlainTextInputElement::new("leave_request_reason_input".into())
                    .with_multiline(true)
                    .into()
            )
            .with_block_id("leave_request_reason".into())
            .with_optional(true)
        )
    ];

    let leave_view_req = SlackApiViewsOpenRequest::new(
        trigger_id.into(),
        SlackView::Modal(
            SlackModalView::new(pt!("Request a leave"), blocks)
                .with_callback_id("submit_leave_request".into())
                .with_private_metadata(channel.into())
                .with_submit(pt!("Submit")),
        ),
    );

    SLACK_CLIENT
        .open_session(&SLACK_TOKEN)
        .views_open(&leave_view_req)
        .await
        .unwrap();
}

pub(crate) async fn submit_leave_request(
    from_channel: &String,
    to_channel: &String,
    user: &String,
    values: &InteractionStateValues,
) {
    let from = &values["leave_request_from"]["leave_request_from_input"]["selected_date"];
    let until = &values["leave_request_until"]["leave_request_until_input"]["selected_date"];
    let full_or_half =
        &values["leave_request_full_half"]["leave_request_full_half_input"]["selected_option"];
    let reason = &values["leave_request_reason"]["leave_request_reason_input"]["value"];
    let reason_str: String = match reason.is_null() {
        true => "".into(),
        false => reason.as_str().unwrap().into(),
    };
    let full_or_half_object = full_or_half.as_object().unwrap();
    let full_or_half_str = &full_or_half_object["value"];

    // send leave request
    let ts = send_leave_request(
        to_channel,
        user,
        &from.as_str().unwrap().into(),
        &until.as_str().unwrap().into(),
        &full_or_half_str.as_str().unwrap().into(),
        match reason_str.len() > 0 {
            true => Some(&reason_str),
            false => None,
        },
    )
    .await;

    // notify managers
    let permalink = get_message_link(&config::<String>("slack.channels.leaves"), &ts).await;
    let managers = config::<Vec<String>>("managers");
    for manager in managers.iter() {
        send_text_message(
            manager.into(),
            &format!(
                "<@{}> has submitted a request for leave. Please <{}|click here> to approve/reject.",
                user, permalink
            )
            .to_string(),
            None,
            None,
        )
        .await;
    }

    // notify requester
    let _ = send_text_message(
        from_channel,
        &"Your request for leave has been submitted for approval.".to_string(),
        None,
        Some(user),
    );
}

pub(crate) async fn update_leave_request(
    channel: &String,
    user: &String,
    from: &String,
    until: &String,
    full_or_half: &String,
    reason: &String,
    manager: &String,
    approved: bool,
    ts: &String,
) {
    // add reaction
    add_reaction(
        channel,
        ts,
        &match approved {
            true => "white_check_mark",
            false => "x",
        }
        .to_string(),
    )
    .await;

    // add reply
    let action = match approved {
        true => "approved",
        false => "rejected",
    };
    send_text_message(
        channel,
        &format!("<@{}> has *{}* this request for leave.", manager, action).to_string(),
        Some(ts),
        None,
    )
    .await;

    // notify requester
    let message = format!(
        "Your request for leave from *{}* to *{}* has been *{}* by <@{}>.",
        from, until, action, manager
    );
    send_text_message(user, &message, None, None).await;

    // remove actions
    let update_chat_req = SlackApiChatUpdateRequest::new(
        SlackChannelId::new(channel.into()),
        SlackMessageContent::new().with_blocks(slack_blocks![
            some_into(SlackSectionBlock::new().with_text(md!(format!(
                "<@{}> has submitted a request for leave from *{}* to *{}*.",
                user, from, until
            )))),
            some_into(
                SlackSectionBlock::new()
                    .with_text(md!(format!("*Full or half day:* {}", full_or_half)))
            ),
            some_into(SlackSectionBlock::new().with_text(md!(format!("*Reason:* _{}_", reason))))
        ]),
        SlackTs(ts.into()),
    );
    let _ = SLACK_CLIENT
        .open_session(&SLACK_TOKEN)
        .chat_update(&update_chat_req)
        .await;
}

pub(crate) async fn verify_request(
    body: &String,
    secret: &String,
    timestamp: &String,
    signature: &String,
) {
    let verifier = SlackVerifier::new(secret.as_str()).unwrap();
    verifier.verify(timestamp, body, signature).unwrap();
}
