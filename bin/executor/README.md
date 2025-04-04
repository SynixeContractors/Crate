# Executor

The executor is a service that is responsible for executing the majority of logic. Not allowed to connect to PostgreSQL directly.

## Connections

NATS  

## Architecture

### Example: Checking for a new missions from GitHub

[Executor:Scheduler] Emits event every 20 minutes  
↓  
[Executor:Handler] Fetches JSON from GitHub, parses  
↓  
[DB] Stores the missions in the database

### Example: Post about upcoming missions

[Executor:Scheduler] Emits event every 5 minutes  
↓  
[Executor:Handler] Requests scheduled missions from the database  
↓  
[DB] Fetches scheduled missions  
↓  
[Executor:Handler] Determines if a notification should be sent  
↓  
[Bot] Sends a notification to the Discord channel  
