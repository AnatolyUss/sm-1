use mysql_async::prelude::*;

#[derive(Debug, PartialEq, Eq, Clone)]
struct Payment {
    customer_id: i32,
    amount: i32,
    account_name: Option<String>,
}

#[tokio::main(flavor = "multi_thread", worker_threads = 2)]
async fn main() {
    let payments = vec![
        Payment {
            customer_id: 1,
            amount: 2,
            account_name: None,
        },
        Payment {
            customer_id: 3,
            amount: 4,
            account_name: Some("foo".into()),
        },
        Payment {
            customer_id: 5,
            amount: 6,
            account_name: None,
        },
        Payment {
            customer_id: 7,
            amount: 8,
            account_name: None,
        },
        Payment {
            customer_id: 9,
            amount: 10,
            account_name: Some("bar".into()),
        },
    ];

    let database_url = "mysql://user_root:secret@localhost:3306/postgresql";
    let pool = mysql_async::Pool::new(database_url);
    let mut conn = pool.get_conn().await.unwrap();

    // Create a temporary table
    r"CREATE TABLE IF NOT EXISTS payment (
        customer_id int not null,
        amount int not null,
        account_name text
    )"
    .ignore(&mut conn)
    .await
    .unwrap();

    // Save payments
    r"INSERT INTO payment (customer_id, amount, account_name)
      VALUES (:customer_id, :amount, :account_name)"
        .with(payments.iter().map(|payment| {
            params! {
                "customer_id" => payment.customer_id,
                "amount" => payment.amount,
                "account_name" => payment.account_name.as_ref(),
            }
        }))
        .batch(&mut conn)
        .await
        .unwrap();

    // Load payments from the database. Type inference will work here.
    let loaded_payments = "SELECT customer_id, amount, account_name FROM payment"
        .with(())
        .map(&mut conn, |(customer_id, amount, account_name)| Payment {
            customer_id,
            amount,
            account_name,
        })
        .await
        .unwrap();

    // Dropped connection will go to the pool
    drop(conn);

    // The Pool must be disconnected explicitly because
    // it's an asynchronous operation.
    pool.disconnect().await.unwrap();

    assert_eq!(loaded_payments, payments);
}
