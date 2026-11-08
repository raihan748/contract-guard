use contract_guard::model::*;
use contract_guard::parser::parse_spec;

#[test]
fn test_parse_openapi_3_0_json() {
    let spec = r##"{
        "openapi": "3.0.3",
        "info": {
            "title": "User Service API",
            "version": "1.0.0"
        },
        "paths": {
            "/api/v1/users": {
                "get": {
                    "summary": "List users",
                    "parameters": [
                        {
                            "name": "limit",
                            "in": "query",
                            "required": false,
                            "schema": { "type": "integer", "default": 20 }
                        }
                    ],
                    "responses": {
                        "200": {
                            "description": "Success",
                            "content": {
                                "application/json": {
                                    "schema": {
                                        "type": "array",
                                        "items": { "$ref": "#/components/schemas/User" }
                                    }
                                }
                            }
                        }
                    }
                },
                "post": {
                    "summary": "Create user",
                    "requestBody": {
                        "required": true,
                        "content": {
                            "application/json": {
                                "schema": { "$ref": "#/components/schemas/CreateUserRequest" }
                            }
                        }
                    },
                    "responses": {
                        "201": { "description": "Created" }
                    }
                }
            }
        },
        "components": {
            "schemas": {
                "User": {
                    "type": "object",
                    "required": ["id", "username", "email"],
                    "properties": {
                        "id": { "type": "string", "format": "uuid" },
                        "username": { "type": "string" },
                        "email": { "type": "string", "format": "email" },
                        "age": { "type": "integer" },
                        "bio": { "type": "string", "nullable": true },
                        "status": {
                            "type": "string",
                            "enum": ["ACTIVE", "INACTIVE", "SUSPENDED"]
                        }
                    }
                },
                "CreateUserRequest": {
                    "type": "object",
                    "required": ["username", "email"],
                    "properties": {
                        "username": { "type": "string" },
                        "email": { "type": "string" }
                    }
                }
            }
        }
    }"##;

    let contract = parse_spec(spec).expect("Should parse OpenAPI 3.0 spec cleanly");
    assert_eq!(contract.title, "User Service API");
    assert_eq!(contract.format_source, "openapi_3.0");

    // Endpoints
    assert_eq!(contract.endpoints.len(), 2);
    let get_ep = contract.endpoints.get("GET /api/v1/users").expect("GET endpoint present");
    assert_eq!(get_ep.method, "GET");
    assert_eq!(get_ep.parameters.len(), 1);
    assert_eq!(get_ep.parameters[0].name, "limit");
    assert_eq!(get_ep.parameters[0].schema_type, FieldType::Primitive(PrimitiveType::Integer));
    assert!(!get_ep.parameters[0].required);

    let post_ep = contract.endpoints.get("POST /api/v1/users").expect("POST endpoint present");
    assert_eq!(post_ep.method, "POST");
    assert!(post_ep.request_body.as_ref().unwrap().required);
    assert_eq!(post_ep.request_body.as_ref().unwrap().schema_ref.as_deref(), Some("CreateUserRequest"));

    // Entities
    assert_eq!(contract.entities.len(), 2);
    let user_entity = contract.entities.get("User").expect("User entity present");
    assert!(user_entity.required_properties.contains("id"));
    assert!(user_entity.required_properties.contains("username"));
    assert!(user_entity.required_properties.contains("email"));
    assert!(!user_entity.required_properties.contains("age"));

    let bio_prop = user_entity.properties.get("bio").unwrap();
    assert!(bio_prop.nullable);

    let status_prop = user_entity.properties.get("status").unwrap();
    assert_eq!(status_prop.enum_values.as_ref().unwrap(), &["ACTIVE", "INACTIVE", "SUSPENDED"]);
}

#[test]
fn test_parse_openapi_3_1_yaml() {
    let spec = r##"
openapi: 3.1.0
info:
  title: Modern Payments API
  version: 2.0.0
paths:
  /v1/charges/{id}:
    get:
      summary: Retrieve charge
      parameters:
        - name: id
          in: path
          required: true
          schema:
            type: string
      responses:
        '200':
          description: OK
components:
  schemas:
    Charge:
      type: object
      required:
        - id
        - amount
      properties:
        id:
          type: string
        amount:
          type: integer
        currency:
          type: string
        receipt_url:
          type:
            - string
            - 'null'
"##;

    let contract = parse_spec(spec).expect("Should parse OpenAPI 3.1 YAML");
    assert_eq!(contract.title, "Modern Payments API");
    assert_eq!(contract.format_source, "openapi_3.1");

    let ep = contract.endpoints.get("GET /v1/charges/{id}").unwrap();
    assert_eq!(ep.parameters[0].name, "id");
    assert_eq!(ep.parameters[0].location, ParameterLocation::Path);
    assert!(ep.parameters[0].required);

    let charge_entity = contract.entities.get("Charge").unwrap();
    let receipt_prop = charge_entity.properties.get("receipt_url").unwrap();
    assert!(receipt_prop.nullable);
}

#[test]
fn test_parse_json_schema_draft7() {
    let schema = r##"{
        "$schema": "http://json-schema.org/draft-07/schema#",
        "title": "CustomerProfile",
        "type": "object",
        "required": ["accountId", "tier"],
        "properties": {
            "accountId": { "type": "string" },
            "tier": { "type": "string", "enum": ["STANDARD", "PRO", "ENTERPRISE"] },
            "score": { "type": "number" },
            "verified": { "type": "boolean" }
        },
        "definitions": {
            "Address": {
                "type": "object",
                "required": ["city", "country"],
                "properties": {
                    "city": { "type": "string" },
                    "country": { "type": "string" }
                }
            }
        }
    }"##;

    let contract = parse_spec(schema).expect("Should parse JSON Schema draft-07");
    assert_eq!(contract.title, "CustomerProfile");
    assert_eq!(contract.format_source, "json_schema_draft7");

    let root = contract.entities.get("CustomerProfile").unwrap();
    assert_eq!(root.properties.len(), 4);
    assert!(root.required_properties.contains("accountId"));
    assert!(root.required_properties.contains("tier"));

    let def_address = contract.entities.get("Address").unwrap();
    assert!(def_address.required_properties.contains("city"));
    assert!(def_address.required_properties.contains("country"));
}

#[test]
fn test_parse_json_schema_2020_12() {
    let schema = r##"{
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "title": "InventoryItem",
        "type": "object",
        "required": ["sku", "stock"],
        "properties": {
            "sku": { "type": "string" },
            "stock": { "type": "integer" },
            "metadata": {
                "type": ["object", "null"]
            }
        },
        "$defs": {
            "Tag": {
                "type": "string",
                "enum": ["SALE", "CLEARANCE", "FEATURED"]
            }
        }
    }"##;

    let contract = parse_spec(schema).expect("Should parse JSON Schema 2020-12");
    assert_eq!(contract.title, "InventoryItem");
    assert_eq!(contract.format_source, "json_schema_2020_12");

    let item = contract.entities.get("InventoryItem").unwrap();
    let meta = item.properties.get("metadata").unwrap();
    assert!(meta.nullable);

    let tag = contract.entities.get("Tag").unwrap();
    assert_eq!(tag.enum_variants.as_ref().unwrap(), &["SALE", "CLEARANCE", "FEATURED"]);
}
