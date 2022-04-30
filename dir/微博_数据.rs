use serde_derive;
use chrono::prelude::*;

#[crud_table]
#[derive(Default, Debug, Clone, PartialEq, serde_derive::Serialize, serde_derive::Deserialize)]
pub struct 微博数据 {
	pub tenantid: Option<String>,
	pub weibo_id: String,
	pub 昵称: Option<String>,
	pub 关注数: Option<i64>,
	pub 粉丝数: Option<i64>,
	pub 作品数: Option<i64>,
	pub 统计截止时间: Date,
}

// ***************************************以下是自定义代码区域******************************************