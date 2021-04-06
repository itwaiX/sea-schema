use std::rc::Rc;
#[cfg(feature="sqlx-mysql")]
use sqlx::{Row, mysql::MySqlRow};
use sea_query::{Expr, Iden, Order, Query, SelectStatement};
use super::{InformationSchema, SchemaQueryBuilder};

#[derive(Debug, sea_query::Iden)]
/// Ref: https://dev.mysql.com/doc/refman/8.0/en/information-schema-key-column-usage-table.html
pub enum KeyColumnUsageFields {
    ConstraintSchema,
    ConstraintName,
    TableSchema,
    TableName,
    ColumnName,
    OrdinalPosition,
    PositionInUniqueConstraint,
    ReferencedTableSchema,
    ReferencedTableName,
    ReferencedColumnName,
}

#[derive(Debug, sea_query::Iden)]
/// Ref: https://dev.mysql.com/doc/refman/8.0/en/information-schema-referential-constraints-table.html
pub enum ReferentialConstraintsFields {
    ConstraintSchema,
    ConstraintName,
    UniqueConstraintSchema,
    UniqueConstraintName,
    UpdateRule,
    DeleteRule,
    TableName,
    ReferencedTableName,
}

#[derive(Debug)]
pub struct ConstraintQueryResult {
    pub constraint_name: String,
    pub column_name: String,
    pub referenced_table_name: String,
    pub referenced_column_name: String,
    pub update_rule: String,
    pub delete_rule: String,
}

impl SchemaQueryBuilder {
    pub fn query_constraints(&self, schema: Rc<dyn Iden>, table: Rc<dyn Iden>) -> SelectStatement {
        type Schema = InformationSchema;
        type Key = KeyColumnUsageFields;
        type Ref = ReferentialConstraintsFields;
        Query::select()
            .columns(vec![
                (Schema::KeyColumnUsage, Key::ConstraintName),
                (Schema::KeyColumnUsage, Key::ColumnName),
                (Schema::KeyColumnUsage, Key::ReferencedTableName),
                (Schema::KeyColumnUsage, Key::ReferencedColumnName),
            ])
            .columns(vec![
                (Schema::ReferentialConstraints, Ref::UpdateRule),
                (Schema::ReferentialConstraints, Ref::DeleteRule),
            ])
            .from((Schema::Schema, Schema::KeyColumnUsage))
            .inner_join(
                (Schema::Schema, Schema::ReferentialConstraints),
                Expr::tbl(Schema::KeyColumnUsage, Key::ConstraintSchema)
                    .equals(Schema::ReferentialConstraints, Ref::ConstraintSchema)
                    .and(
                        Expr::tbl(Schema::KeyColumnUsage, Key::ConstraintName)
                            .equals(Schema::ReferentialConstraints, Ref::ConstraintName)
                    )
            )
            .and_where(Expr::tbl(Schema::KeyColumnUsage, Key::ConstraintSchema).eq(schema.to_string()))
            .and_where(Expr::tbl(Schema::KeyColumnUsage, Key::TableName).eq(table.to_string()))
            .order_by(Key::ConstraintName, Order::Asc)
            .order_by(Key::OrdinalPosition, Order::Asc)
            .take()
    }
}

#[cfg(feature="sqlx-mysql")]
impl From<&MySqlRow> for ConstraintQueryResult {
    fn from(row: &MySqlRow) -> Self {
        Self {
            constraint_name: row.get(0),
            column_name: row.get(1),
            referenced_table_name: row.get(2),
            referenced_column_name: row.get(3),
            update_rule: row.get(4),
            delete_rule: row.get(5),
        }
    }
}