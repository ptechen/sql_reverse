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
#[allow(non_snake_case)]
#[derive(Serialize, Clone, Default, Debug)]
pub struct Field {
    /// 数据库字段名
    pub field_name: String,
    /// 首字母大写驼峰字段名
    pub FieldName: String,
    /// 首字母小写驼峰字段名
    pub fieldName: String,
    /// 数据库类型类型
    pub database_field_type: String,
    /// 字段类型
    pub field_type: String,
    /// 注释
    pub comment: String,
    /// 默认值是否为null, 1: 是 0: 不是
    pub is_null: u8,
    /// 默认值
    pub default: Option<String>,
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

impl Table {
    pub async fn skip_fields(&self, skip_fields: Vec<String>) -> Table {
        let mut fields = vec![];
        for field in self.fields.iter() {
            let mut flag = true;
            for keys in &self.index_key {
                if keys.contains(&field.field_name) {
                    fields.push(field.to_owned());
                    flag = false;
                    continue;
                }
            }
            if !skip_fields.contains(&field.field_name) && flag {
                fields.push(field.to_owned());
            }
        }
        Table {
            table_name: self.table_name.to_owned(),
            struct_name: self.struct_name.to_owned(),
            fields,
            comment: self.comment.to_owned(),
            index_key: self.index_key.to_owned(),
        }
    }

    pub async fn contain_fields(&self, contain_fields: Vec<String>) -> Table {
        let mut fields = vec![];
        for field in self.fields.iter() {
            let mut flag = true;
            for keys in &self.index_key {
                if keys.contains(&field.field_name) {
                    fields.push(field.to_owned());
                    flag = false;
                    continue;
                }
            }
            if contain_fields.contains(&field.field_name) && flag{
                fields.push(field.to_owned());
            }
        }
        Table {
            table_name: self.table_name.to_owned(),
            struct_name: self.struct_name.to_owned(),
            fields,
            comment: self.comment.to_owned(),
            index_key: self.index_key.to_owned(),
        }
    }
}
