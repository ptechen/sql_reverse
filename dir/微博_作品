use serde_derive;
use chrono::prelude::*;

#[crud_table]
#[derive(Default, Debug, Clone, PartialEq, serde_derive::Serialize, serde_derive::Deserialize)]
pub struct 微博作品 {
	pub tenantid: Option<String>,
	pub weibo_id: Option<String>,
	pub 作品ID: String,
	pub 标题: Option<String>,
	pub 点赞量: Option<i64>,
	pub 转发量: Option<i64>,
	pub 评论量: Option<i64>,
	pub 创建时间: Option<NaiveDateTime>,
	pub 作品类型: Option<String>,
	pub 统计截止时间: Date,
}

// ***************************************以下是自定义代码区域******************************************