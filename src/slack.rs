use crate::config;
use slack_morphism::prelude::*;

pub(crate) async fn add_reaction(channel: String, ts: String, reaction: String) {
    let client = SlackClient::new(SlackClientHyperConnector::new().unwrap());
    let token_value: SlackApiTokenValue = config::<String>("slack.bot_token").into();
    let token: SlackApiToken = SlackApiToken::new(token_value);
    let session = client.open_session(&token);
    let reaction_add_req = SlackApiReactionsAddRequest::new(
        SlackChannelId::new(channel),
        SlackReactionName::new(reaction),
        ts.into(),
    );

    let _ = session.reactions_add(&reaction_add_req).await;
}

pub(crate) async fn get_message_link(channel: String, ts: String) -> String {
    let client = SlackClient::new(SlackClientHyperConnector::new().unwrap());
    let token_value: SlackApiTokenValue = config::<String>("slack.bot_token").into();
    let token: SlackApiToken = SlackApiToken::new(token_value);
    let session = client.open_session(&token);
    let chat_permalink_req =
        SlackApiChatGetPermalinkRequest::new(SlackChannelId::new(channel), ts.into());
    let chat_permalink_resp = session
        .chat_get_permalink(&chat_permalink_req)
        .await
        .unwrap();

    chat_permalink_resp.permalink.to_string()
}

pub(crate) async fn send_leave_request(
    channel: String,
    user: String,
    from: String,
    until: String,
    reason: Option<String>,
) -> String {
    let blocks: Vec<SlackBlock> = slack_blocks![
        some_into(SlackSectionBlock::new().with_text(md!(format!(
            "<@{}> has submitted a leave request from `{}` to `{}`.",
            user, from, until
        )))),
        some_into(SlackSectionBlock::new().with_text(md!(format!(
            "*Reason:* {}",
            reason.or(Some("No reason provided.".to_string())).unwrap()
        )))),
        some_into(SlackActionsBlock::new(slack_blocks![
            some_into(
                SlackBlockButtonElement::new("approve_leave_request".into(), pt!("Approve"),)
                    .with_confirm(SlackBlockConfirmItem::new(
                        pt!("Really?"),
                        pt!("This will mark the leave as approved. It cannot be undone."),
                        pt!("Approve"),
                        pt!("Cancel")
                    ))
                    .with_value(format!("{}|{}|{}", user, from, until).into())
                    .with_style("primary".into())
            ),
            some_into(
                SlackBlockButtonElement::new("reject_leave_request".into(), pt!("Reject"),)
                    .with_confirm(SlackBlockConfirmItem::new(
                        pt!("Really?"),
                        pt!("This will mark the leave as reject. It cannot be undone."),
                        pt!("Reject"),
                        pt!("Cancel")
                    ))
                    .with_value(format!("{}|{}|{}", user, from, until).into())
                    .with_style("danger".into())
            )
        ],))
    ];

    let client = SlackClient::new(SlackClientHyperConnector::new().unwrap());
    let token_value: SlackApiTokenValue = config::<String>("slack.bot_token").into();
    let token: SlackApiToken = SlackApiToken::new(token_value);
    let session = client.open_session(&token);
    let post_chat_req = SlackApiChatPostMessageRequest::new(
        SlackChannelId::new(channel),
        SlackMessageContent::new().with_blocks(blocks),
    );
    let post_chat_resp = session.chat_post_message(&post_chat_req).await.unwrap();

    post_chat_resp.ts.to_string()
}

pub(crate) async fn show_leave_form_view(channel: String, trigger_id: String) {
    let blocks: Vec<SlackBlock> = slack_blocks![
        some_into(
            SlackSectionBlock::new()
                .with_text(pt!("Fill and submit below information to request a leave."))
        ),
        some_into(
            SlackInputBlock::new(
                pt!("From"),
                SlackBlockDatePickerElement::new("leave_request_from_input".into()).into()
            )
            .with_block_id("leave_request_from".into())
        ),
        some_into(
            SlackInputBlock::new(
                pt!("Until"),
                SlackBlockDatePickerElement::new("leave_request_until_input".into()).into()
            )
            .with_block_id("leave_request_until".into())
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

    let client = SlackClient::new(SlackClientHyperConnector::new().unwrap());
    let token_value: SlackApiTokenValue = config::<String>("slack.bot_token").into();
    let token: SlackApiToken = SlackApiToken::new(token_value);
    let session = client.open_session(&token);
    let leave_view_req = SlackApiViewsOpenRequest::new(
        trigger_id.into(),
        SlackView::Modal(
            SlackModalView::new(pt!("Request a leave"), blocks)
                .with_callback_id("submit_leave_request".into())
                .with_private_metadata(channel)
                .with_submit(pt!("Submit")),
        ),
    );

    session.views_open(&leave_view_req).await.unwrap();
}

pub(crate) async fn send_text_message(
    channel: String,
    msg: String,
    ts: Option<String>,
    user: Option<String>,
) -> Option<String> {
    let client = SlackClient::new(SlackClientHyperConnector::new().unwrap());
    let token_value: SlackApiTokenValue = config::<String>("slack.bot_token").into();
    let token: SlackApiToken = SlackApiToken::new(token_value);
    let session = client.open_session(&token);
    return if user.is_some() {
        let post_ephemeral_req = SlackApiChatPostEphemeralRequest::new(
            channel.into(),
            user.as_ref().unwrap().into(),
            SlackMessageContent::new().with_text(msg),
        );

        let _ = session.chat_post_ephemeral(&post_ephemeral_req).await;

        None
    } else {
        let mut post_chat_req = SlackApiChatPostMessageRequest::new(
            channel.into(),
            SlackMessageContent::new().with_text(msg),
        )
        .with_unfurl_links(false);

        if let Some(thread_ts) = ts {
            post_chat_req = post_chat_req.with_thread_ts(SlackTs::new(thread_ts));
        }

        let post_chat_resp = session.chat_post_message(&post_chat_req).await.unwrap();

        Some(post_chat_resp.ts.to_string())
    };
}
