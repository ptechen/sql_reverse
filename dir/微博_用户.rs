use serde_derive;
use chrono::prelude::*;

#[crud_table]
#[derive(Default, Debug, Clone, PartialEq, serde_derive::Serialize, serde_derive::Deserialize)]
pub struct 微博用户 {
	pub tenantid: Option<String>,
	pub weibo_id: String,
	pub 昵称: Option<String>,
	pub avatar: Option<String>,
	pub 生日: Option<String>,
	pub 星座: Option<String>,
	pub 公司名: Option<String>,
	pub 公司性质: Option<String>,
	pub 创建时间: Option<String>,
	pub 官方认证: Option<String>,
	pub 描述: Option<String>,
	pub 性别: Option<String>,
	pub 地址: Option<String>,
	pub 信用: Option<String>,
	pub 标签: Option<String>,
	pub 统计截止时间: Option<NaiveDateTime>,
}

// ***************************************以下是自定义代码区域******************************************