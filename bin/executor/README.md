# Executor

The executor is a service that is responsible for executing the majority of logic. Not allowed to connect to PostgreSQL directly.

## Connections

NATS  

## Architecture

### Example: Checking for a new missions from GitHub

[Scheduler] Emits event every 20 minutes  
↓  
[Executor] Fetches JSON from GitHub, parses  
↓  
[DB] Stores the missions in the database

### Example: Post about upcoming missions

[Scheduler] Emits event every 5 minutes  
↓  
[Executor] Requests scheduled missions from the database  
↓  
[DB] Fetches scheduled missions  
↓  
[Executor] Determines if a notification should be sent  
↓  
[Bot] Sends a notification to the Discord channel  
