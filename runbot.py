#!/usr/bin/env python3
from discord.ext import commands

bot = commands.Bot(command_prefix='-')

# Type hint that url should be a str. This doesn't perform a cast, though.
def split_url(url: str):
    '''Splits a url of the format https://discord.com/channels/{ch}/{th}/{id} into {ch}, {th}, {id}.'''
    # Cast url to a str. This won't cause any errors, everything can be str-cast.
    url = str(url)
    # Finds the index of the first occurence of this prefix, or -1.
    startind = url.find("https://discord.com/channels/")
    # The index will not be -1 if and only if the prefix was found.
    if startind != -1:
        # The string was found. Only the part after that prefix is useful to us.
        subpart = url[startind+len("https://discord.com/channels/"):]
        # Let's check that there are at least two slashes in the rest of the string.
        if subpart.count("/") >= 2:
            # Okay, let's split it on slashes, and take the first three.
            ch, th, id = subpart.split("/")[0:3]
            # Check that all of these are actually nonnegative integers.
            if ch.isdigit() and (int(ch)>= 0) and th.isdigit() and (int(th) >= 0) and id.isdigit() and (int(id) >= 0):
                # Okay, looks valid. Let's return.
                return [int(ch), int(th), int(id)]
            else:
                # Some of these are not valid positive ints. Error value: 1.
                return 1
        else:
            # They don't have enough slashes. Error value: 2.
            return 2
    else:
        # The prefix wasn't found. This wasn't a url. Error value: 3.
        return 3

# This passes our first argument through the parser+splitter.
@commands.command()
async def pin(ctx, split: split_url):
    # Handle our error cases first.
    if split == 3:
        # TODO: Handle non-url.
        return
    if split == 2:
        # TODO: Handle insufficient slashes.
        return
    if split == 1:
        # TODO: Handle invalid positive ints.
        return
    # Okay, we're past the errors. Unpack that array.
    ch, th, id = split
    try:
        # Get the channel with this id.
        # Note: must be in the same server as the caller.
        chObj = ctx.guild.fetch_channel(ch)
    except:
        # TODO: Handle invalid channel id.
        return
    # Check if it's a TextChannel. TODO: make sure this works.
    if not isinstance(chObj, TextChannel)
        # TODO: Handle non-text channel.
        return
    # Check if the thread id is different from the channel id.
    if ch != th:
        try:
            # We're in a thread.
            thObj = chObj.get_thread(th)
        except:
            # TODO: Handle invalid thread id.
            return
    else:
        # We aren't in a thread, just a TextChannel.
        thObj = chObj
    # Now for the message itself.
    try:
        idObj = thObj.fetch_message(id)
    except:
        # TODO: Handle invalid message id.
        return
    # Pin the message.
    try:
        idObj.pin()
    except:
        # TODO: Handle insufficient perms.
        return
