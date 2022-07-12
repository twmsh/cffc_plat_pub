#!/bin/bash

rm -f ../deploy/cfbm.db

sqlite3 ../deploy/cfbm.db <sqlite3_init.sql

echo "done."
