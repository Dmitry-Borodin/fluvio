
mod create;
mod delete;
mod describe;
mod list;
mod helpers;

use structopt::StructOpt;

use create::CreateTopicOpt;
use delete::DeleteTopicOpt;
use describe::DescribeTopicsOpt;
use list::ListTopicsOpt;

use create::process_create_topic;
use delete::process_delete_topic;
use describe::process_describe_topics;
use list::process_list_topics;

use crate::Terminal;

use super::CliError;

#[derive(Debug, StructOpt)]
#[structopt(name = "topic", author = "", about = "Topic operations")]
pub enum TopicOpt {
    #[structopt(name = "create", author = "", template = "{about}

{usage}

{all-args}
",about = "Create a topic")]
    Create(CreateTopicOpt),

    #[structopt(name = "delete", author = "", template = "{about}

{usage}

{all-args}
",about = "Delete a topic")]
    Delete(DeleteTopicOpt),

    #[structopt(name = "describe", author = "", template = "{about}

{usage}

{all-args}
",about = "Show details of a topic")]
    Describe(DescribeTopicsOpt),

    #[structopt(name = "list", author = "", template = "{about}

{usage}

{all-args}
",about = "Show all topics")]
    List(ListTopicsOpt),
}

pub(crate) async fn process_topic<O>(out: std::sync::Arc<O>,topic_opt: TopicOpt) -> Result<String, CliError>
    where O: Terminal
{
    match topic_opt {
        TopicOpt::Create(create_topic_opt) => process_create_topic(create_topic_opt).await,
        TopicOpt::Delete(delete_topic_opt) => process_delete_topic(delete_topic_opt).await,
        TopicOpt::Describe(describe_topics_opt) => process_describe_topics(out,describe_topics_opt).await,
        TopicOpt::List(list_topics_opt) => process_list_topics(out,list_topics_opt).await
    }
}
