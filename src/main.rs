use clap::{crate_authors, crate_description, crate_name, crate_version, App, Arg};
use malipo::{
    AccountsMemStore, CsvDataReader, CsvWriterStdout, Fallible, PaymentsEngine,
    TransactionsMemStore,
};

fn main() -> Fallible<()> {
    let matches = App::new(crate_name!())
        .version(crate_version!())
        .author(crate_authors!())
        .about(crate_description!())
        .arg(
            Arg::with_name("INPUT")
                .help("Sets the input file to use")
                .required(true)
                .index(1),
        )
        .get_matches();
    let input_fname = matches.value_of("INPUT").unwrap();
    let transactions = CsvDataReader::new(input_fname)?;

    let acc_store = Box::new(AccountsMemStore::new());
    let txn_store = Box::new(TransactionsMemStore::new());
    let mut engine = PaymentsEngine::new(acc_store, txn_store);
    for txn in transactions {
        engine.execute_transaction(txn?)?;
    }
    CsvWriterStdout::write(engine.accounts()?, Some(std::io::stdout()))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    macro_rules! tst {
        ($name:ident, $input:expr, $expected:expr) => {
            #[test]
            fn $name() -> Fallible<()> {
                let mut input_file = NamedTempFile::new()?;
                input_file.write_all($input.as_bytes())?;
                let txns = CsvDataReader::new(input_file.path().to_str().unwrap())?;
                let acc_store = Box::new(AccountsMemStore::new());
                let txn_store = Box::new(TransactionsMemStore::new());
                let mut engine = PaymentsEngine::new(acc_store, txn_store);

                for txn in txns {
                    engine.execute_transaction(txn?)?;
                }
                let mut output = vec![];
                CsvWriterStdout::write(engine.accounts()?, Some(&mut output))?;
                let data = String::from_utf8(output)?;
                assert_eq!(data, $expected);
                Ok(())
            }
        };
    }
    tst!(
        test_deposit,
        "type,client,tx,amount\ndeposit,2,12,1.77\ndeposit,2,13, 1.77",
        "client,available,held,total,locked\n2,3.5400,0.0000,3.5400,false\n"
    );

    tst!(
        test_dispute,
        "type,client,tx,amount\ndeposit,2,12,1.77\ndispute,2,12\ndeposit,2,13, 1.77\ndeposit,2,14, 1.77",
        "client,available,held,total,locked\n2,3.5400,1.7700,5.3100,false\n"
    );

    tst!(
        test_chargeback,
        "type,client,tx,amount\ndeposit,1,1,100.1\nchargeback,1,1\ndispute,1,1\nchargeback,1,1",
        "client,available,held,total,locked\n1,0.0000,0.0000,0.0000,true\n"
    );

    tst!(
       test_resolution,
       "type,client,tx,amount\ndeposit,1,1,100.1\ndispute,1,1\ndeposit,2,12,1.77\ndispute,2,12\nresolve,2,12\nresolve,2,12\nresolve,2,12\nresolve,2,12",
       "client,available,held,total,locked\n1,0.0000,100.1000,100.1000,false\n2,1.7700,0.0000,1.7700,false\n"
   );

    tst!(
        test_scenario_1,
        "type,client,tx,amount
      deposit,1,1,1.0
      deposit,1,3,2.0
      withdrawal,1,5,1.5
      dispute,1,3
      deposit,2,2,2.0
      withdrawal,2,4,3.0",
        "client,available,held,total,locked\n1,-0.5000,2.0000,1.5000,false\n2,2.0000,0.0000,2.0000,false\n"
    );
}
