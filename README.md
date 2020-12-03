# JSON to PYI (WIP)
json2pyi infers and generates Python type definitions (dataclass or TypedDict) from a sample JSON file.

## Example
**Input:**
```json
{
    "page": {
        "id": "kctbh9vrtdwd",
        "name": "GitHub",
        "url": "https://www.githubstatus.com",
        "time_zone": "Etc/UTC",
        "updated_at": "2020-12-03T08:11:21.385Z"
    },
    "components": [
        {
            "id": "8l4ygp009s5s",
            "name": "Git Operations",
            "status": "operational",
            "created_at": "2017-01-31T20:05:05.370Z",
            "updated_at": "2020-10-29T22:51:43.831Z",
            "position": 1,
            "description": "Performance of git clones, pulls, pushes, and associated operations",
            "showcase": false,
            "start_date": null,
            "group_id": null,
            "page_id": "kctbh9vrtdwd",
            "group": false,
            "only_show_if_degraded": false
        },
        /* ... */
    ],
    "incidents": [],
    "scheduled_maintenances": [],
    "status": {
        "indicator": "none",
        "description": "All Systems Operational"
    }
}
```

**Output:**
```python
@dataclass
class Page:
    id = str
    name = str
    url = str
    time_zone = str
    updated_at = str

@dataclass
class Components:
    id = str
    name = str
    status = str
    created_at = str
    updated_at = str
    position = int
    description = Optional[str]
    showcase = bool
    start_date = None
    group_id = None
    page_id = str
    group = bool
    only_show_if_degraded = bool

@dataclass
class Status:
    indicator = str
    description = str

@dataclass
class RootObject:
    page = Page
    components = List[Components]
    incidents = List[Any]
    scheduled_maintenances = List[Any]
    status = Status
```

## TODO
- [ ] Detect tuple (array)
- [ ] Detect UUID / datetime
- [ ] Detect Enum
- [ ] Merge data types with similar structure and common name prefix/suffix
- [ ] Detect recursive type definition (e.g. tree) 
- [ ] Include imports of non-primitive types
- [ ] Generate TypedDict
- [ ] Refactor to unify TypedDict and dataclass generation
- [ ] Compile to WASM and provide a Web-based app
- [ ] Avoid merge data types with totally different structures in a union
- [ ] Avoid unnecessary heap allocation by reducing one-time usage of Vec 
- [ ] Allow specifying the order of generated data types 

## Credits
The project is inspired by: 
- https://app.quicktype.io/?l=ts
- https://github.com/thautwarm/schema-provider.py
- https://jvilk.com/MakeTypes/
