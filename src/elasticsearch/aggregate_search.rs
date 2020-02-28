use crate::elasticsearch::{Elasticsearch, ElasticsearchError};
use crate::zdbquery::ZDBQuery;
use serde::de::DeserializeOwned;
use serde::export::PhantomData;
use serde::*;
use serde_json::*;

pub struct ElasticsearchAggregateSearchRequest<ReturnType>
where
    ReturnType: DeserializeOwned,
{
    elasticsearch: Elasticsearch,
    query: ZDBQuery,
    agg_json: serde_json::Value,
    _marker: PhantomData<ReturnType>,
}

impl<ReturnType> ElasticsearchAggregateSearchRequest<ReturnType>
where
    ReturnType: DeserializeOwned,
{
    pub fn new(
        elasticsearch: &Elasticsearch,
        query: ZDBQuery,
        agg_json: serde_json::Value,
    ) -> ElasticsearchAggregateSearchRequest<ReturnType> {
        ElasticsearchAggregateSearchRequest::<ReturnType> {
            elasticsearch: elasticsearch.clone(),
            query,
            agg_json,
            _marker: PhantomData::<ReturnType>,
        }
    }

    pub fn execute(self) -> std::result::Result<ReturnType, ElasticsearchError> {
        let mut url = self.elasticsearch.base_url();
        url.push_str("/_search");
        url.push_str("?size=0");
        let query_dsl = self.query.query_dsl().expect("zdbquery has no query_dsl");
        let client = reqwest::Client::new()
            .get(&url)
            .header("content-type", "application/json")
            .body(
                serde_json::to_string(&json! {
                    {
                        "query": query_dsl,
                        "aggs": {
                            "the_agg": self.agg_json
                        }
                    }
                })
                .unwrap(),
            );

        Elasticsearch::execute_request(client, |_, body| {
            #[derive(Deserialize)]
            struct Shards {
                total: u32,
                failed: u32,
                skipped: u32,
                successful: u32,
            }

            #[derive(Deserialize)]
            struct TheAgg {
                the_agg: serde_json::Value,
            }

            #[derive(Deserialize)]
            struct AggregateResponse {
                #[serde(rename = "_shards")]
                shards: Shards,
                aggregations: TheAgg,
            }

            let agg_resp: AggregateResponse =
                serde_json::from_str(&body).expect("received invalid aggregate json response");
            let the_agg = agg_resp.aggregations.the_agg;
            let foo: ReturnType = serde_json::from_value(the_agg).unwrap();
            Ok(foo)
        })
    }
}