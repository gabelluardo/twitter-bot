# Twitter-bot

Simple bot that keeps synchronized a blog wiht rss feed and a twitter account.

All [twitter-developer](https://developer.twitter.com) credentials are enviroment variables stored in `.env` file.  

``` sh
RSS=
USER_ID=
CONSUMER_KEY=
CONSUMER_SECRET=
ACCESS_TOKEN=
ACCESS_TOKEN_SECRET=
```

or via cli 

``` 
twitter-bot 
    --access-token <access-token> \
    --access-token-secret <access-token-secret> \
    --bearer-token <bearer-token> \
    --consumer-key <consumer-key> \
    --consumer-secret <consumer-secret> \
    --rss <rss> \
    --user-id <user-id>
```

`RSS` is the url of the blog feed

## Author

**twitter-bot** © [gabelluardo](https://github.com/gabelluardo)  
Released under the [MIT](https://github.com/gabelluardo/twitter-bot/blob/master/LICENSE) License.
