use std::fmt;
use std::default::Default;
use crypto::sha2::Sha256;
use jwt::{Header,Token};
use rocket::Outcome;
use rocket::http::{Status,RawStr};
use rocket::request::{self, Request, FromRequest, FromFormValue};
use bson::oid::ObjectId;
use service::ServiceError;
use redis;

#[derive(PartialEq, Debug, Serialize, Deserialize,Clone)]
pub struct Order {
    #[serde(rename = "_id")]
    pub id :String,
    pub openid: String,
    pub trip_id: String,
    pub trip_owner: String,
    pub order_id: Option<String>,   //微信支付参数 
    pub transaction_id: Option<String>,//微信支付参数 
    pub tel: Option<String>,
    pub status: OrderStatus,
    pub price: i64,
    pub count:i64,
    pub start_time: i64
}



#[derive(PartialEq, Debug, Serialize, Deserialize,Default,Clone)]
pub struct Trip {
    #[serde(rename = "_id")]
    pub id :String,
    pub openid: String,
    pub seat_count : i64,
    pub current_seat : i64,
    pub start_time : i64,
    pub start:String,
    pub end:String,
    pub price:i64,
    pub venue:String, //出发地点
    pub status:TripStatus,
    pub message:Option<String>,
    pub plate_number: String,
    pub tel: String,
    pub car_type: String,
}

#[derive(FromForm)]
pub struct TripForm {
    pub seat_count : i64,
    pub start_time : i64,
    pub start:String,
    pub end:String,
    pub price:i64,
    pub venue:String, //集合地点
    pub message:Option<String>,
    pub plate_number: String,
    pub car_type: String,
    pub tel: String,
}

#[derive(PartialEq, Debug, Serialize, Deserialize,Clone)]
pub enum TripStatus {
    Prepare,
    Full,
    Running,
    Finish,
    Cancel
}

#[derive(PartialEq, Debug, Serialize, Deserialize,Default,Clone)]
pub struct Complain {
    #[serde(rename = "_id")]
    pub id :Option<String>,
    pub openid: String,
    pub content: String
}

//weixin api result
#[derive(PartialEq, Debug, Serialize, Deserialize)]
pub struct ApiResult {
    pub access_token: Option<String>,
    pub expires_in:Option<i64>,
    pub errcode:Option<i64>,
    pub errmsg:Option<String>,
    pub refresh_token:Option<String>,
    pub openid:Option<String>,
    pub ticket:Option<String>,
    pub scope:Option<String>
}

#[derive(PartialEq, Debug, Serialize, Deserialize,Clone)]
pub struct WxUserInfo {
    openid:String,
    nickname: String,
    sex:i64,
    language:String,
    city:String,
    province:String,
    country:String,
    headimgurl: String,
    unionid:Option<String>
}

#[derive(PartialEq, Debug, Serialize, Deserialize,Clone)]
pub enum OrderStatus {
    Unpaid,
    Paid,
    Submit,
}

#[derive(Default, RustcDecodable, RustcEncodable, Debug)]
pub struct JwtUser {
    pub id : String,   //ID 如果是微信登录就是openid
	pub name : String,
	pub role : String,
	pub user_type: String,
    pub exp : i64,
}

impl JwtUser {
	pub fn from_jwt(s: &str) -> Option<Self> {
		let token = Token::<Header, JwtUser>::parse(s).unwrap();
        println!("{:?}", token);
		 if token.verify(b"geekgogo", Sha256::new()) {
	        Some(token.claims)
	    } else {
	        None
	    }
	}
}

impl<'a, 'r> FromRequest<'a, 'r> for JwtUser {
    type Error = ServiceError;

    fn from_request(request: &'a Request<'r>) -> request::Outcome<JwtUser, ServiceError> {
        let keys: Vec<_> = request.headers().get("Authorization").collect();
        if keys.len() != 1 {
            return Outcome::Failure((Status::Unauthorized, ServiceError::NoAuth));
        }
        let (_, key) = keys[0].split_at(7);
        if let Some(user) = JwtUser::from_jwt(key) {
            if user.user_type == "weixin" {
                Outcome::Success(user)
            } else {
                Outcome::Failure((Status::Unauthorized, ServiceError::NoAuth))
            }
        } else {
        	Outcome::Failure((Status::Unauthorized, ServiceError::NoAuth))
        }
    }
}

impl Default for TripStatus {
    fn default() -> TripStatus {
        TripStatus::Prepare
    }
}

impl Default for OrderStatus {
    fn default() -> OrderStatus {
        OrderStatus::Unpaid
    }
}

impl fmt::Display for OrderStatus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            OrderStatus::Unpaid => write!(f, "Unpaid"),
            OrderStatus::Paid => write!(f, "Paid"),
            OrderStatus::Submit => write!(f, "Submit"),
        }
    }
}
impl fmt::Display for TripStatus {
    fn fmt(&self, f:&mut fmt::Formatter) -> fmt::Result {
        match *self {
            TripStatus::Prepare => write!(f, "Prepare"),
            TripStatus::Running => write!(f,"Running"),
            TripStatus::Finish => write!(f, "Finish"),
            TripStatus::Full => write!(f, "Full"),
            TripStatus::Cancel => write!(f,"Cancel")
        }
    }
}

impl<'a> redis::ToRedisArgs for &'a TripStatus {
    fn to_redis_args(&self) -> Vec<Vec<u8>> {
        vec![format!("{}",self).into_bytes()]
    }
}

impl<'a> redis::ToRedisArgs for &'a OrderStatus {
    fn to_redis_args(&self) -> Vec<Vec<u8>> {
        vec![format!("{}",self).into_bytes()]
    }
}

impl redis::FromRedisValue for OrderStatus {
    fn from_redis_value(v: &redis::Value) -> redis::RedisResult<Self> {
        if let redis::Value::Data(ref data) = *v {
            let s = String::from_utf8_lossy(&data);
            match &*s {
                "Unpaid" => Ok(OrderStatus::Unpaid),
                "Paid" => Ok(OrderStatus::Paid),
                "Submit" => Ok(OrderStatus::Submit),
                _ => Ok(OrderStatus::Unpaid),
            }
        } else {
            Err(redis::RedisError::from((redis::ErrorKind::TypeError,"not a Data")))
        }
    }
}

impl Trip {
    pub fn new(openid:String,form:TripForm) -> Trip{
        Trip{
            openid,
            tel:form.tel,
            id:ObjectId::new().unwrap().to_hex(),
            seat_count:form.seat_count,
            current_seat:form.seat_count,
            start_time:form.start_time,
            start:form.start,
            end:form.end,
            price:form.price,
            venue:form.venue,
            status:TripStatus::Prepare,
            message:form.message,
            plate_number:form.plate_number,
            car_type:form.car_type,
        }
    }
}

impl Order {
    pub fn new(trip:Trip, openid:String,count:i64,tel:Option<String>) -> Self {
        Order{
            id:ObjectId::new().unwrap().to_hex(),
            trip_id:trip.id,
            trip_owner:trip.openid,
            openid,
            order_id:None,
            transaction_id:None,
            tel,
            status:OrderStatus::Unpaid,
            count,
            price:trip.price,
            start_time:trip.start_time,
        }
    }
}


impl<'t> FromFormValue<'t> for OrderStatus {
    type Error = ServiceError;

    fn from_form_value(from_value: &'t RawStr) -> Result<OrderStatus,ServiceError> {
         match from_value.as_str() {
            "Unpaid" => Ok(OrderStatus::Unpaid),
            "Paid" => Ok(OrderStatus::Paid),
            "Submit" => Ok(OrderStatus::Submit),
            _ => Err(ServiceError::String("error user type".to_owned()))
        }
    }
}

impl<'t> FromFormValue<'t> for TripStatus {
    type Error = ServiceError;

    fn from_form_value(from_value: &'t RawStr) -> Result<TripStatus,ServiceError> {
         match from_value.as_str() {
            "Prepare" => Ok(TripStatus::Prepare),
            "Full" => Ok(TripStatus::Full),
            "Running" => Ok(TripStatus::Running), 
            "Finish" => Ok(TripStatus::Finish), 
            "Cancel" => Ok(TripStatus::Cancel),
            _ => Ok(TripStatus::Finish) 
        }
    }
}


    
