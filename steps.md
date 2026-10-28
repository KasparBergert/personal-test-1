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

---

user adds a certificate, receives alerts if its expired.
same way prometheus alertmanager fires alerts.

there are states for certificates.
firing
on edge
ok

firing is that there is an issue with the certificate.
on edge means there is about to be an issue if it says for longer
ok means no issues.

---

adapter method

wont really work as i expected.

every day it should look through all the certificates. see which ones are about to expire.
filter it by user.
create specific, on-edge, firing emails.
send the emails
keep track

this is a crontab or a seperate job thing that will start.
do not use crontabs. implement laravels job.

a job is called every day at 1 am. your comptuer keeps track. runs every second on another thread
...

solution.

start a thread that sleeps most of the program. it wakes up every minute to check if its 3 am.
if so,
it launches a sequence of events.
every day it should look through all the certificates. see which ones are about to expire. filter it by user.
create specific, on-edge, firing emails.
send the emails
keep track of who received the emails lately, so emails are not sent again the next day
