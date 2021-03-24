# JSON to Python Types
*json2pyi* infers a type schema from a sample JSON file and generates Python type definitions ([`dataclass`](https://docs.python.org/3/library/dataclasses.html), [`BaseModel`](https://pydantic-docs.helpmanual.io/usage/models/) or [`TypedDict`](https://www.python.org/dev/peps/pep-0589/)) accordingly.

<!--Even though the project is still an MVP, it is expected to be stable & usable as a Web app. Please do not hesitate to raise an issue if you find any problems.-->

__Now available online__: https://json2pyi.pages.dev

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
- [x] Detect UUID / datetime
- [ ] Detect Enum
- [x] Merge data types with similar structure and common name prefix/suffix
- [x] Detect recursive type definition (e.g. tree) 
- [x] Include imports of non-primitive types
- [x] Generate type alias for complex Union
- [x] Generate TypedDict
- [ ] <del>Refactor to unify TypedDict and dataclass generation</del>
- [x] Compile to WASM and provide a Web-based app
- [ ] Allow to control more options on Web app (partially blocked by https://github.com/vhiribarren/raytracer-rust/issues/8) 
- [ ] Avoid merge data types with totally different structures in a union
- [ ] Avoid unnecessary heap allocation by reducing one-time usage of Vec 
- [ ] Allow specifying the order of generated data types 
- [ ] Support more input types, such as JSON Schema
- [ ] Support more target languages

## Credits
The project is inspired by: 
- https://app.quicktype.io/?l=ts
- https://github.com/thautwarm/schema-provider.py
- https://jvilk.com/MakeTypes/
- https://github.com/koxudaxi/datamodel-code-generator/
