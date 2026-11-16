DELETE FROM epg_guides WHERE url IS NOT NULL AND id NOT IN (SELECT min(id) FROM epg_guides WHERE url IS NOT NULL GROUP BY url);
CREATE UNIQUE INDEX epg_guides_url ON epg_guides(url);
