UPDATE edits SET edits_json = json_set(edits_json, '$.masks', (
    SELECT json_group_array(
        CASE WHEN type = 'object' THEN json_remove(value, '$.color') ELSE value END ORDER BY key)
    FROM json_each(edits_json, '$.masks')))
WHERE json_type(edits_json, '$.masks') = 'array'
    AND EXISTS (
        SELECT 1 FROM json_each(edits_json, '$.masks')
        WHERE type = 'object' AND json_type(value, '$.color') IS NOT NULL);

UPDATE edits_history SET edits_json = json_set(edits_json, '$.masks', (
    SELECT json_group_array(
        CASE WHEN type = 'object' THEN json_remove(value, '$.color') ELSE value END ORDER BY key)
    FROM json_each(edits_json, '$.masks')))
WHERE json_type(edits_json, '$.masks') = 'array'
    AND EXISTS (
        SELECT 1 FROM json_each(edits_json, '$.masks')
        WHERE type = 'object' AND json_type(value, '$.color') IS NOT NULL);

UPDATE presets SET manifest_json = json_set(manifest_json, '$.ops.masks.layers', (
    SELECT json_group_array(
        CASE WHEN type = 'object' THEN json_remove(value, '$.color') ELSE value END ORDER BY key)
    FROM json_each(manifest_json, '$.ops.masks.layers')))
WHERE json_type(manifest_json, '$.ops.masks.layers') = 'array'
    AND EXISTS (
        SELECT 1 FROM json_each(manifest_json, '$.ops.masks.layers')
        WHERE type = 'object' AND json_type(value, '$.color') IS NOT NULL);
