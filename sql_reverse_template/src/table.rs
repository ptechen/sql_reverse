use crate::render::Render;
use serde::Serialize;

/// sql 表
#[derive(Serialize, Clone, Default, Debug)]
pub struct Table {
    /// 表名
    pub table_name: String,
    /// 结构体名
    pub struct_name: String,
    /// 字段列表
    pub fields: Vec<Field>,
    /// 表注释
    pub comment: String,
    /// 索引
    pub index_key: Vec<Vec<String>>,
}

/// sql 字段
#[derive(Serialize, Clone, Default, Debug)]
pub struct Field {
    /// 字段名
    pub field_name: String,
    /// 字段类型
    pub field_type: String,
    /// 注释
    pub comment: String,
    /// 默认值是否为null, 1: 是 0: 不是
    pub is_null: u8,
}

impl Table {
    pub fn new(
        table_name: String,
        struct_name: String,
        fields: Vec<Field>,
        comment: String,
    ) -> Table {
        Table {
            table_name,
            struct_name,
            fields,
            comment,
            index_key: vec![],
        }
    }
}

impl Render for Table {}
