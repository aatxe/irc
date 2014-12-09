(function() {var implementors = {};
implementors['openssl'] = ["<a class='stability Unmarked' title='No stability level'></a>impl <a class='trait' href='http://doc.rust-lang.org/nightly/std/io/trait.Reader.html' title='std::io::Reader'>Reader</a> for <a class='struct' href='openssl/bio/struct.MemBio.html' title='openssl::bio::MemBio'>MemBio</a>","<a class='stability Unmarked' title='No stability level'></a>impl&lt;S: <a class='trait' href='http://doc.rust-lang.org/nightly/std/io/trait.Stream.html' title='std::io::Stream'>Stream</a>&gt; <a class='trait' href='http://doc.rust-lang.org/nightly/std/io/trait.Reader.html' title='std::io::Reader'>Reader</a> for <a class='struct' href='openssl/ssl/struct.SslStream.html' title='openssl::ssl::SslStream'>SslStream</a>&lt;S&gt;","<a class='stability Unmarked' title='No stability level'></a>impl&lt;S&gt; <a class='trait' href='http://doc.rust-lang.org/nightly/std/io/trait.Reader.html' title='std::io::Reader'>Reader</a> for <a class='enum' href='openssl/ssl/enum.MaybeSslStream.html' title='openssl::ssl::MaybeSslStream'>MaybeSslStream</a>&lt;S&gt;",];
implementors['irc'] = ["<a class='stability Experimental' title='Experimental'></a>impl <a class='trait' href='http://doc.rust-lang.org/nightly/std/io/trait.Reader.html' title='std::io::Reader'>Reader</a> for <a class='enum' href='irc/conn/enum.NetStream.html' title='irc::conn::NetStream'>NetStream</a>",];

            if (window.register_implementors) {
                window.register_implementors(implementors);
            } else {
                window.pending_implementors = implementors;
            }
        
})()
