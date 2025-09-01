# rust-payment-engine

 Simple Payment Engine

 CLI tool that simulates a Payment Engine
 Receiving transaction data in CSV format, and output final Client Accounts to Std Out (in CSV Format)

 ## Usage:
 ```
 cargo run -- transactions.csv > accounts.csv
 ```
 
 ## CSV Input File:
 Input CSV must have the following fields: type of transaction, client ID, transaction ID, amount. E.g.:
 ```
 type, client, tx, amount
 deposit, 1, 1, 2.0
 withdrawal, 1, 2, 1.5
 ```
 
 ## Supported Transaction Types:
 - Deposit: Increase funds
 - Withdrawal: Decrease Available funds, if enough
 - Dispute: Mark transaction for reversal investigation. (Done by tx, amount not needed)
 - Resolve: Resolves dispute, making funds available. (Done by tx, amount not needed)
 - Chargeback: Withdraw funds under dispute. Account is locked afterwards. (Done by tx, amount not needed)
 
 ## Implementation

 1 - Based on format and types of transactions, created a simple project structure with transactions and accounts as domain items

 2 - As the project is intended to have a CSV as input, created a simple csv_processor module (using 'csv' lib & documentation)

 3 - Used structs with 'serde' lib to process CSV input files, and created same for eventual 'Client Account' output format

 4 - Considering that the input data can be very large, and for controlling disputes the previous transactions must be easy to consult, decided to use a simple database (SQLite in this case) to avoid storing the data in memory, and also have access to already processed inputs. Also, seemed reasonable to keep the history of transactions

 5 - Found project 'serde_rusqlite' to simplify usage between 'csv <-> serde <-> sqlite'

 6 - Created the transaction logic, processing each input row at a time (considering chronological order)

 7 - Tested manually, using a simple input and output CSVs for initial validation

 8 - Used Copilot to create Unit Tests for the main logic (handle each Transaction Type), due to time constraints

 9 - Used Copilot to create large CSVs and integration test.

 10 - Added deletion of SQLite files at the end of each execution, to make re-testing easier

 ## AI Usage

 Implementation was done in VSCode, using Copilot Integration.
 Main development was done normally, using Copilot for autofilling and fix suggestion on compiling errors, and DuckDuckGo + DeepSeek for doubts and examples.
 Copilot queries were used for the following: 
 - Create Unit Tests with Mocking: "Create multiple unit tests to test all scenarios from process_transaction"
 - Create CSVs to use for integration testing: "Create a CSV with 200 lines that covers all transaction scenarios from process_transaction, and create an output CSV so the results can be compared"
 - Create Integration test with large CSVs: "In tests dir, create a test file that uses the big_input.csv file as input, and the compares its result with big_output.csv"
 - Check the code for issues and improvements: It suggested including a custom serialization to limit to 4 decimal places.

# TODO:

Modify the code to allow parallel concurrent CSV used as inputs, simulating a large demand environment.
Initial ideas would be to implement queues to receive all demands, and use async processing.
Would need to identify a reasonable way of ensuring chronolical order or processing, considering dates are not provided, and requests can be out of order.
