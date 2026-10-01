1. crud for creating certificates

2. auth for logging in

3. cron that sends emails when certificate is expired.

set up actix-web

1. you can run it and call it with curl

2. add authentication.

user gives his info.
save the info into a database.

implement login
search the user in the db.
respond ok or deny

end:
user can register
user can login
propert testing for both endpoints

unit, integration, end-to-end

database lib.
sqlite

tables
user
-name
-password
