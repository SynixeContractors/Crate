import psycopg2

import backpacks
import vests
import helmets
import uniforms

conn = psycopg2.connect(
    host="prod-do-user-6857898-0.b.db.ondigitalocean.com",
    port="25060",
    database="synixe",
    user="doadmin",
    password="vTrX0on6sAaoJWpd")

if __name__ == "__main__":
    cur = conn.cursor()
    for group in [["Backpack", backpacks.generate_classes()]]: #[["Vests", vests.generate_classes()], ["Headwear", helmets.generate_classes()], ["Uniform", uniforms.generate_classes()]]:
        for item in group[1]:
            print("{} - {}".format(group[0], item[0]))
            cur.execute("""
INSERT INTO gear_items (class, roles, category, global, enabled)
VALUES (%s, %s, %s, %s, %s)
""", (item[0], "{}", group[0], "false", "true"))
            cur.execute("""
INSERT INTO gear_cost (class, cost, priority)
VALUES (%s, %s, %s)
""", (item[0], item[1], 0))
        conn.commit()
