use std::fmt;
use std::default::Default;
use crypto::sha2::Sha256;
use jwt::{Header,Token};
use rocket::Outcome;
use rocket::http::{Status,RawStr};
use rocket::request::{self, Request, FromRequest, FromFormValue};
use bson::Bson;
use bson::oid::ObjectId;
use service::ServiceError;

//车主
#[derive(PartialEq, Debug, Serialize, Deserialize,Clone,FromForm)]
pub struct User {
	#[serde(rename = "_id")]
    pub id :Option<String>,
    pub openid: String,
    pub tel:String,
    pub name: String,
    pub card_id: String,
    pub plate_number:Option<String>,
    pub car_type : Option<String>,
    pub car_pic : Option<String>,
    pub refund_count:i32,
    pub user_type : UserType
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
    pub price: i32,
    pub count:i32,
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
    pub id :Option<String>,
    pub openid: String,
    pub seat_count : i32,
    pub current_seat : i32,
    pub start_time : i64,
    pub start_time_text : String,
    pub line_id:i32,
    pub start:String,
    pub end:String,
    pub price:i32,
    pub venue:String,
    pub status:String,
    pub level:i32,
    pub message:Option<String>,
    pub plate_number: String,
    pub tel: Option<String>,
    pub car_type: String,
    pub orders:Vec<Order>
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
    pub expires_in:Option<i32>,
    pub errcode:Option<i32>,
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
    sex:i32,
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
	name : String,
	role : String,
	user_type: String
}

impl JwtUser {
	pub fn from_jwt(s: &str) -> Option<Self> {
		let token = Token::<Header, JwtUser>::parse(s).unwrap();
		 if token.verify(b"geekgogo", Sha256::new()) {
	        Some(token.claims)
	    } else {
	        None
	    }
	}
}

impl<'a, 'r> FromRequest<'a, 'r> for JwtUser {
    type Error = ();

    fn from_request(request: &'a Request<'r>) -> request::Outcome<JwtUser, ()> {
        let keys: Vec<_> = request.headers().get("Authorization").collect();
        if keys.len() != 1 {
            return Outcome::Failure((Status::BadRequest, ()));
        }
        let (_, key) = keys[0].split_at(7);

        if let Some(user) = JwtUser::from_jwt(key) {
        	 Outcome::Success(user)
        } else {
        	Outcome::Failure((Status::BadRequest, ()))
        }
    }
}


impl Default for UserType {
    // add code here
    fn default() -> UserType {
        UserType::Anonymous
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

#[derive(PartialEq, Debug, Serialize, Deserialize,Clone,Default)]
pub struct LoginStatus {
    pub user_type : UserType,
    pub openid : String,
    pub name : Option<String>,
    pub web_token : Option<String>,
    pub refresh_token : Option<String>,
    pub user:Option<User>,
    pub code:Option<i32>
}

impl User {
    pub fn new_owner(tel:String,name:String,card_id:String,car_type:String,car_pic:String,plate_number:String,openid:String) -> User {
       User{id:None,
                car_type:Some(car_type),
                car_pic:Some(car_pic),
                tel:tel,
                name:name,
                card_id:card_id,
                plate_number:Some(plate_number),
                openid:openid,
                refund_count:3,
                user_type:UserType::Owner}
    }
    pub fn new_passenger(tel:String,name:String,card_id:String,openid:String) -> User {
       User{id:None,
                car_type:None,
                car_pic:None,
                tel:tel,
                name:name,
                card_id:card_id,
                plate_number:None,
                openid:openid,
                refund_count:3,
                user_type:UserType::Passenger}
    }
}


impl Order {
    
    pub fn set_status(&mut self,status:OrderStatus) {
        self.status = format!("{}",status);
    }
    
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

    pub fn add_order(&mut self,o:Order) {
        //todo 
        print!("{:?}", o);
    }

	pub fn buy_seats(&mut self,count:i32) -> bool {
	    if self.current_seat >= count {
		    self.current_seat -= count;
            true
	    } else {
            false
	    }
	}

    pub fn check_orders_done(&self) -> bool {
    	//todo 
        false
    }

    pub fn set_all_status(&self,status:OrderStatus) {
        //todo 
        print!("{}", status);
    }

    pub fn has_order(&self,openid:&str) -> bool {
        //todo
        print!("{}", openid);
        false 
    }

    pub fn set_status(&mut self, status:TripStatus) {
        self.status = format!("{}",status);
    }

    pub fn get_status(&self) -> TripStatus {
        match self.status.as_str() {
            "Prepare" => TripStatus::Prepare,
            "Full" => TripStatus::Full,
            "Running" => TripStatus::Running, 
            "Finish" => TripStatus::Finish, 
            "Cancel" => TripStatus::Cancel,
            _ => TripStatus::Finish 
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


    
