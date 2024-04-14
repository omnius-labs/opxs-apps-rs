// https://docs.aws.amazon.com/ses/latest/dg/notification-contents.html

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct SesNotification {
    pub notification_type: String,
    pub mail: Mail,
    pub bounce: Option<Bounce>,
    pub complaint: Option<Complaint>,
    pub delivery: Option<Delivery>,
}

#[derive(Debug, Deserialize)]
pub struct Mail {
    pub timestamp: String,
    pub message_id: String,
    pub source: String,
    pub source_arn: String,
    pub source_ip: String,
    pub sending_account_id: String,
    pub caller_identity: String,
    pub destination: Vec<String>,
    pub headers_truncated: bool,
    pub headers: Vec<Header>,
    pub common_headers: CommonHeaders,
}

#[derive(Debug, Deserialize)]
pub struct Header {
    pub name: String,
    pub value: String,
}

#[derive(Debug, Deserialize)]
pub struct CommonHeaders {
    pub from: Vec<String>,
    pub date: String,
    pub to: Vec<String>,
    pub message_id: String,
    pub subject: String,
}

#[derive(Debug, Deserialize)]
pub struct Bounce {
    pub bounce_type: String,
    pub bounce_subtype: String,
    pub bounced_recipients: Vec<BouncedRecipient>,
    pub timestamp: String,
    pub feedback_id: String,
    pub remote_mta_ip: String,
    pub reporting_mta: String,
}

#[derive(Debug, Deserialize)]
pub struct BouncedRecipient {
    pub email_address: String,
    pub action: String,
    pub status: String,
    pub diagnostic_code: String,
}

#[derive(Debug, Deserialize)]
pub struct Complaint {
    pub complained_recipients: Vec<ComplainedRecipient>,
    pub timestamp: String,
    pub feedback_id: String,
    pub complaint_sub_type: String,
    pub user_agent: String,
    pub complaint_feedback_type: String,
    pub arrival_date: String,
}

#[derive(Debug, Deserialize)]
pub struct ComplainedRecipient {
    pub email_address: String,
}

#[derive(Debug, Deserialize)]
pub struct Delivery {
    pub timestamp: String,
    pub processing_time_millis: i64,
    pub recipients: Vec<String>,
    pub smtp_response: String,
    pub reporting_mta: String,
    pub remote_mta_ip: String,
}
