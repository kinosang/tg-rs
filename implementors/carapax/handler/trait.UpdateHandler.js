(function() {var implementors = {};
implementors["carapax_access"] = [{text:"impl&lt;P&gt; <a class=\"trait\" href=\"carapax/handler/trait.UpdateHandler.html\" title=\"trait carapax::handler::UpdateHandler\">UpdateHandler</a> for <a class=\"struct\" href=\"carapax_access/struct.AccessHandler.html\" title=\"struct carapax_access::AccessHandler\">AccessHandler</a>&lt;P&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;P: <a class=\"trait\" href=\"carapax_access/trait.AccessPolicy.html\" title=\"trait carapax_access::AccessPolicy\">AccessPolicy</a>,&nbsp;</span>",synthetic:false,types:["carapax_access::handler::AccessHandler"]},];
implementors["carapax_ratelimit"] = [{text:"impl <a class=\"trait\" href=\"carapax/handler/trait.UpdateHandler.html\" title=\"trait carapax::handler::UpdateHandler\">UpdateHandler</a> for <a class=\"struct\" href=\"carapax_ratelimit/struct.DirectRateLimitHandler.html\" title=\"struct carapax_ratelimit::DirectRateLimitHandler\">DirectRateLimitHandler</a>",synthetic:false,types:["carapax_ratelimit::direct::DirectRateLimitHandler"]},{text:"impl&lt;K&gt; <a class=\"trait\" href=\"carapax/handler/trait.UpdateHandler.html\" title=\"trait carapax::handler::UpdateHandler\">UpdateHandler</a> for <a class=\"struct\" href=\"carapax_ratelimit/struct.KeyedRateLimitHandler.html\" title=\"struct carapax_ratelimit::KeyedRateLimitHandler\">KeyedRateLimitHandler</a>&lt;K&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;K: <a class=\"trait\" href=\"carapax_ratelimit/trait.RateLimitKey.html\" title=\"trait carapax_ratelimit::RateLimitKey\">RateLimitKey</a>,&nbsp;</span>",synthetic:false,types:["carapax_ratelimit::keyed::KeyedRateLimitHandler"]},];

            if (window.register_implementors) {
                window.register_implementors(implementors);
            } else {
                window.pending_implementors = implementors;
            }
        
})()
