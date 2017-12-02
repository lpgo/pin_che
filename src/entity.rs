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

//车主
#[derive(PartialEq, Debug, Serialize, Deserialize,Clone,FromForm)]
pub struct User {
	#[serde(rename = "_id")]
    pub openid :String,
    pub tel:String,
    pub name: String,
    pub card_id: String,
    pub plate_number:String,
    pub car_type : String,
    pub car_pic : String,
    pub refund_count:i64,
    pub user_type : UserType
}

#[derive(FromForm)]
pub struct OwnerForm {
    pub tel:String,
    pub card_id: String,
    pub plate_number:String,
    pub car_type : String,
    pub car_pic : String,
    pub code : String,  //短信验证码
}

#[derive(PartialEq, Debug, Serialize, Deserialize,Clone)]
pub struct Admin {
   #[serde(rename = "_id")]
    pub id :Option<String>,
    pub name:String,
    pub pwd:String
}
#[derive(PartialEq, Debug, Serialize, Deserialize,Clone)]
pub struct Order {
    #[serde(rename = "_id")]
    pub id :Option<String>,
    pub openid: String,
    pub trip_id: String,
    pub order_id: String,
    pub transaction_id: String,
    pub tel: Option<String>,
    pub status: String,
    pub price: i64,
    pub count:i64,
    pub start_time: i64
}

#[derive(PartialEq, Debug, Serialize, Deserialize,Default,Clone)]
pub struct Complain {
    #[serde(rename = "_id")]
    pub id :Option<String>,
    pub openid: String,
    pub content: String
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
    pub status:TripStatus,
    pub message:Option<String>,
    pub plate_number: String,
    pub car_type: String,
}

#[derive(PartialEq, Debug, Serialize, Deserialize,Clone)]
pub enum TripStatus {
    Prepare,
    Full,
    Running,
    Finish,
    Cancel
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
pub enum UserType {
    Owner,
    Passenger,
    Anonymous
}

#[derive(PartialEq, Debug, Serialize, Deserialize,Clone)]
pub enum OrderStatus {
    PaySuccess,
    PayFail,
    Submit,
    Refund,
    Request     //request refund
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


impl Default for UserType {
    // add code here
    fn default() -> UserType {
        UserType::Anonymous
    }
}

impl Default for TripStatus {
    // add code here
    fn default() -> TripStatus {
        TripStatus::Prepare
    }
}


impl fmt::Display for UserType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            UserType::Owner => write!(f, "Owner"),
            UserType::Passenger => write!(f, "Passenger"),
            UserType::Anonymous => write!(f, "Anonymous")
        }
    }
}

impl fmt::Display for OrderStatus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            OrderStatus::PaySuccess => write!(f, "PaySuccess"),
            OrderStatus::PayFail => write!(f, "PayFail"),
            OrderStatus::Submit => write!(f, "Submit"),
            OrderStatus::Request => write!(f, "Request"),
            OrderStatus::Refund => write!(f, "Refund")
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

#[derive(PartialEq, Debug, Serialize, Deserialize,Clone,Default)]
pub struct LoginStatus {
    pub user_type : UserType,
    pub openid : String,
    pub name : Option<String>,
    pub web_token : Option<String>,
    pub refresh_token : Option<String>,
    pub user:Option<User>,
    pub code:Option<i64>
}

impl User {
    pub fn new_owner(openid:String,name:String,owner:OwnerForm) -> User {
       User{openid,
        name,
        tel:owner.tel,
        card_id:owner.card_id,
        plate_number:owner.plate_number,
        car_type:owner.car_type,
        car_pic:owner.car_pic,
        user_type:UserType::Owner,
        refund_count:2}
    }
}


impl Order {
    pub fn get_status(&self) -> OrderStatus {
        match self.status.as_str() {
            "PayFail" => OrderStatus::PayFail,
            "PaySuccess" => OrderStatus::PaySuccess,
            "Submit" => OrderStatus::Submit,
            "Refund" => OrderStatus::Refund,
            "Request" => OrderStatus::Request,
            _ => OrderStatus::PayFail
        }
    }
}

impl Trip {

    pub fn new(openid:String,tel:String,form:TripForm) -> Trip{
        Trip{
            openid,
            tel,
            id:ObjectId::new().unwrap().to_hex(),
            seat_count:form.seat_count,
            current_seat:0,
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


impl<'t> FromFormValue<'t> for UserType {
    type Error = ServiceError;

    fn from_form_value(from_value: &'t RawStr) -> Result<UserType,ServiceError> {
         match from_value.as_str() {
            "Owner" => Ok(UserType::Owner),
            "Passenger" => Ok(UserType::Passenger),
            "Anonymous" => Ok(UserType::Anonymous),
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


    
