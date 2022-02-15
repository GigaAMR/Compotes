use crate::structs::operation_state::OperationState;
use rusqlite::named_params;
use rusqlite::Connection;
use serde::Deserialize;
use serde::Serialize;
use serde_rusqlite::from_rows;

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct Operation {
    pub(crate) id: u32,
    pub(crate) operation_date: String,
    pub(crate) op_type: String,
    pub(crate) type_display: String,
    pub(crate) details: String,
    pub(crate) amount_in_cents: i32,
    pub(crate) hash: String,
    pub(crate) state: OperationState,
    pub(crate) ignored_from_charts: bool,
    pub(crate) bank_account_id: u32,
    #[serde(deserialize_with = "crate::serialization::deserialize_tags_ids::deserialize_tags_ids")]
    pub(crate) tags_ids: Vec<u32>,
}

pub(crate) fn find_all(conn: &Connection) -> Vec<Operation> {
    let mut stmt = conn
        .prepare(
            "
        SELECT
            id,
            operation_date,
            type as op_type,
            type_display,
            details,
            amount_in_cents,
            hash,
            state,
            bank_account_id,
            ignored_from_charts,
            (
                SELECT GROUP_CONCAT(tag_id)
                FROM operation_tag
                WHERE operation_id = operations.id
            ) AS tags_ids
        FROM operations
        ORDER BY operation_date DESC
    ",
        )
        .expect("Could not fetch operations");

    let mut operations: Vec<Operation> = Vec::new();

    let mut rows_iter = from_rows::<Operation>(stmt.query([]).unwrap());

    loop {
        match rows_iter.next() {
            None => break,
            Some(result) => {
                let operation = result.expect("Could not deserialize Operation item");
                operations.push(operation);
            }
        }
    }

    operations
}

pub(crate) fn get_by_id(conn: &Connection, id: u32) -> Operation {
    let mut stmt = conn
        .prepare(
            "
        SELECT
            id,
            operation_date,
            type as op_type,
            type_display,
            details,
            amount_in_cents,
            hash,
            state,
            bank_account_id,
            ignored_from_charts,
            (
                SELECT GROUP_CONCAT(tag_id)
                FROM operation_tag
                WHERE operation_id = operations.id
            ) AS tags_ids
        FROM operations
        WHERE id = :id
    ",
        )
        .expect("Could not fetch operation");

    let mut rows = stmt.query(named_params!{
            ":id": &id,
        }).expect("Could not execute query to fetch an operation by id.");

    let row = rows.next().expect("Could not retrieve query rows.").expect("No operation found with this ID.");

    serde_rusqlite::from_row::<Operation>(row).unwrap()
}

pub(crate) fn find_triage(conn: &Connection) -> Vec<Operation> {
    let mut stmt = conn
        .prepare(
            "
        SELECT
            id,
            operation_date,
            type as op_type,
            type_display,
            details,
            amount_in_cents,
            hash,
            state,
            bank_account_id,
            ignored_from_charts,
            (
                SELECT GROUP_CONCAT(tag_id)
                FROM operation_tag
                WHERE operation_id = operations.id
            ) AS tags_ids
        FROM operations
        WHERE state = :triage
        ORDER BY details DESC
    ",
        )
        .expect("Could not fetch operations");

    let mut operations: Vec<Operation> = Vec::new();

    let mut rows_iter = from_rows::<Operation>(stmt.query(named_params! {
        ":triage": OperationState::PendingTriage,
    }).unwrap());

    loop {
        match rows_iter.next() {
            None => break,
            Some(result) => {
                let operation = result.expect("Could not deserialize Operation item");
                operations.push(operation);
            }
        }
    }

    operations
}

pub(crate) fn insert_all(conn: &mut Connection, operations: Vec<Operation>) {
    let mut stmt = conn
        .prepare(
            "
            INSERT INTO operations
            (
                operation_date,
                type,
                type_display,
                details,
                amount_in_cents,
                hash,
                state,
                bank_account_id,
                ignored_from_charts
            )
            VALUES
            (
                :operation_date,
                :type,
                :type_display,
                :details,
                :amount_in_cents,
                :hash,
                :state,
                :bank_account_id,
                :ignored_from_charts
            )
        ",
        )
        .expect("Could not create query to insert operation.");

    for operation in operations.iter() {
        let res = stmt.execute(named_params! {
            ":operation_date": &operation.operation_date,
            ":type": &operation.op_type,
            ":type_display": &operation.type_display,
            ":details": &operation.details,
            ":amount_in_cents": &operation.amount_in_cents,
            ":hash": &operation.hash,
            ":state": &operation.state,
            ":bank_account_id": &operation.bank_account_id,
            ":ignored_from_charts": &operation.ignored_from_charts,
        })
        .expect("Could not insert operation");

        dbg!(&operation, &res);
    }
}

pub(crate) fn refresh_statuses_with_hashes(conn: &mut Connection) -> usize {
    let transaction = conn
        .transaction()
        .expect("Could not create database transaction.");

    let result = {
        let mut stmt = transaction
        .prepare("
        update operations
        set state = :triage
        where operations.state != :triage
        and operations.hash in (
            select t2.hash
            from operations as t2
            group by t2.hash
            having count(t2.hash) > 1
        )
        ",
        )
        .expect("Could not create query to update operations state.");

        stmt.execute(named_params! {
            ":triage": &OperationState::PendingTriage.to_string(),
        })
            .expect("Could not execute update operations state query")
    };

    transaction
        .commit()
        .expect("Could not execute update operations state transaction");

    result
}

pub(crate) fn update_details(conn: &mut Connection, id: String, details: String) {
    let mut stmt = conn
        .prepare("
        update operations
        set details = :details,
        state = :ok
        where id = :id
        ",
        )
        .expect("Could not create query to update operations details.");

    stmt.execute(named_params! {
        ":id": &id,
        ":details": &details,
        ":ok": &OperationState::Ok.to_string(),
    })
        .expect("Could not execute update operations details query");
}

pub(crate) fn delete(conn: &mut Connection, id: String) {
    let mut stmt = conn
        .prepare("delete from operations where id = :id")
        .expect("Could not create query to delete operation.")
        ;

    stmt.execute(named_params! {
        ":id": &id,
    })
        .expect("Could not execute delete operation");
}
