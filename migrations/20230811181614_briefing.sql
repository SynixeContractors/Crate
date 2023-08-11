ALTER TABLE missions ADD COLUMN briefing jsonb;
UPDATE missions SET briefing = jsonb_build_object('old', description);
ALTER TABLE missions ALTER COLUMN briefing SET NOT NULL;
