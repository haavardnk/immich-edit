UPDATE edits SET edits_json = json_set(edits_json, '$.masks', json((
    SELECT json_group_array(
        CASE WHEN type = 'object' THEN json_remove(value, '$.color') ELSE value END ORDER BY key)
    FROM json_each(json(json_extract(edits_json, '$.masks'))))))
WHERE json_type(edits_json, '$.masks') = 'text'
    AND json_valid(json_extract(edits_json, '$.masks'))
    AND json_type(json_extract(edits_json, '$.masks')) = 'array';

UPDATE edits_history SET edits_json = json_set(edits_json, '$.masks', json((
    SELECT json_group_array(
        CASE WHEN type = 'object' THEN json_remove(value, '$.color') ELSE value END ORDER BY key)
    FROM json_each(json(json_extract(edits_json, '$.masks'))))))
WHERE json_type(edits_json, '$.masks') = 'text'
    AND json_valid(json_extract(edits_json, '$.masks'))
    AND json_type(json_extract(edits_json, '$.masks')) = 'array';

UPDATE presets SET manifest_json = json_set(manifest_json, '$.ops.masks.layers',
    json(json_extract(manifest_json, '$.ops.masks.layers')))
WHERE json_type(manifest_json, '$.ops.masks.layers') = 'text'
    AND json_valid(json_extract(manifest_json, '$.ops.masks.layers'))
    AND json_type(json_extract(manifest_json, '$.ops.masks.layers')) = 'array';
