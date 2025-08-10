# PostgreSQL

## Install
`brew install postgresql
`
## start
`brew services start postgresql
`
## service check
`brew services list
`

## Access the psql shell
`psql -U postgres
`
## Create a New User With a Password
`CREATE ROLE admin WITH LOGIN PASSWORD 'qwer1234';
`
## Change Password for an Existing User
`
ALTER USER admin WITH PASSWORD 'qwer1234';
`

## Grant Database Permissions
`
GRANT ALL PRIVILEGES ON DATABASE test TO admin;
`

## [Grammar](./postgre_sql.md)