ALTER TABLE instance_config ADD COLUMN map_style_light TEXT;
ALTER TABLE instance_config ADD COLUMN map_style_dark TEXT;
ALTER TABLE instance_config ADD COLUMN geocoder_url TEXT NOT NULL DEFAULT 'https://photon.komoot.io';
