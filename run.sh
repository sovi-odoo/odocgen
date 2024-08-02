#!/bin/sh
set -e

odoo_root=".."

_run() {
    exec cargo run --release -- -o output -l local "$@"
}

if [ -d "$odoo_root/enterprise" ]
then
    _run "$odoo_root/odoo/odoo/addons" "$odoo_root/odoo/addons" \
        "$odoo_root/enterprise"
else
    _run "$odoo_root/odoo/odoo/addons" "$odoo_root/odoo/addons"
fi
